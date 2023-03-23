// Copyright (C) 2023 Doug Anderson
// SPDX-License-Identifier: MIT

use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use libp2p::{
    core::{muxing::StreamMuxerBox, ConnectedPoint},
    gossipsub, identity,
    multiaddr::{Multiaddr, Protocol},
    ping,
    swarm::{dial_opts::DialOpts, SwarmBuilder, SwarmEvent},
    webrtc, Transport,
};
use log::debug;
use rand::thread_rng;
use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::{mpsc, oneshot};

mod behaviour;
use behaviour::OutEvent;

#[derive(Debug, Clone)]
pub struct ServerResponse {
    pub address: Bytes,
}

pub struct Message<T> {
    pub reply: Responder<T>,
}

type Responder<T> = oneshot::Sender<T>;

/// An example WebRTC server that will accept connections and run the protocols on them.
pub async fn start(mut request_recvr: mpsc::Receiver<Message<ServerResponse>>) -> Result<()> {
    // let mut config: Config = Config::default();
    let config: Arc<Mutex<HashMap<String, Bytes>>> = Arc::new(Mutex::new(HashMap::new()));
    let config_getter = Arc::clone(&config);

    // Spawn an API manager to receive incoming Requests
    tokio::spawn(async move {
        log::debug!("Reply listener spawned");
        fn lock_get(cfg: &Arc<Mutex<HashMap<String, Bytes>>>, key: &str) -> Bytes {
            let lock = cfg.lock().unwrap();
            lock.get(key).unwrap().clone()
        }

        while let Some(message) = request_recvr.recv().await {
            let address = lock_get(&config_getter, "address");
            // ignore any errors
            let _ = message.reply.send(ServerResponse { address });
        }
    });

    let gossipsub_topic = gossipsub::IdentTopic::new("chat");

    let mut swarm = {
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = id_keys.public().to_peer_id();
        let transport = webrtc::tokio::Transport::new(
            id_keys.clone(),
            webrtc::tokio::Certificate::generate(&mut thread_rng())?,
        );

        let transport = transport
            .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)))
            .boxed();

        let mut behaviour = behaviour::SuperChatBehaviour::new(id_keys);

        behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();

        SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build()
    };

    // Listen for connections on the given port.
    let address = Multiaddr::from(Ipv6Addr::UNSPECIFIED)
        .with(Protocol::Udp(0))
        .with(Protocol::WebRTC);

    let _id = swarm.listen_on(address.clone())?;

    loop {
        let config_setter = Arc::clone(&config);

        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr {
                address,
                listener_id: _,
            } => {
                // check if address string contains "::" at all, if so skip the connection prompt
                if !address.to_string().contains("::") {
                    // add p2p PeerId to address as p2p Protocol
                    let full_address = address
                        .with(Protocol::P2p(*swarm.local_peer_id().as_ref()))
                        .to_string();

                    let mut config_setter = config_setter.lock().unwrap();
                    if config_setter.get("address").is_none() {
                        log::debug!(">>>> Setting ADDRESS");

                        config_setter
                            .insert("address".to_owned(), Bytes::from(full_address.clone()));
                    }
                } else {
                    log::info!("Listening on {address}");
                }
            }
            SwarmEvent::IncomingConnection { send_back_addr, .. } => {
                eprintln!("➡️  Incoming Connection on {send_back_addr:?}")
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                endpoint: ConnectedPoint::Listener { send_back_addr, .. },
                established_in,
                ..
            } => {
                eprintln!("✔️  Connection Established to {peer_id} in {established_in:?} on {send_back_addr}");
                // This dial doesn't work, but it's the only way I can get the gossipsub to connect...
                // let mut res = send_back_addr;
                // strip_peer_id(&mut res);

                // eprintln!("📞  Server dialing the browser {res}");

                // let dial_opts = DialOpts::unknown_peer_id()
                //     // .condition(PeerCondition::NotDialing)
                //     .address(res.clone())
                //     // .extend_addresses_through_behaviour()
                //     .build();
                // if let Err(e) = swarm.dial(dial_opts) {
                //     println!("❌  (Expected) Dialing error: {e:?}");
                // }
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id)
            }
            SwarmEvent::Behaviour(OutEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                println!(
                    "📨 Got message: '{}' with id: {id} from peer: {peer_id}",
                    String::from_utf8_lossy(&message.data),
                );
            }
            SwarmEvent::Behaviour(OutEvent::Gossipsub(gossipsub::Event::Subscribed {
                peer_id,
                topic, // : gossipsub::TopicHash { hash },
            })) => {
                println!("💨💨💨  {topic:?} Subscriber: {peer_id}");

                // show peers and stuff
                let p = swarm
                    .behaviour()
                    .gossipsub
                    .all_mesh_peers()
                    .cloned()
                    .collect::<Vec<_>>();

                let num = p.len();

                debug!("### Number peers: {num:?} ### ");

                p.iter().map(|p| println!("Peer: {p:?}")).for_each(drop);

                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);

                let msg = make_msg(&peer_id.to_base58());

                if let Err(e) = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(gossipsub_topic.clone(), msg.as_bytes())
                {
                    println!("❌  Subscriber Publish error: {e:?}, {msg}");
                }
            }
            SwarmEvent::Behaviour(OutEvent::Ping(ping::Event {
                peer,
                result: Ok(ping::Success::Ping { rtt }),
            })) => {
                let id = peer.to_string().to_owned();
                eprintln!("🏐 Pinged {id} ({rtt:?})")
            }
            SwarmEvent::Behaviour(OutEvent::Ping(ping::Event {
                peer,
                result: Ok(ping::Success::Pong),
            })) => {
                let id = peer.to_string().to_owned();
                eprintln!("🏓 Ponged by {id}");

                let msg = make_msg(&peer.to_base58());

                if let Err(e) = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(gossipsub_topic.clone(), msg.as_bytes())
                {
                    println!("❌  Subscriber Publish error: {e:?}, {msg}");
                }
            }
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
                ..
            } => {
                println!("❌👂 Listener Closed on {listener_id:?} b/c {reason:?}");

                for address in addresses.iter() {
                    println!("❌🏠 {address}");
                }
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                swarm
                    .behaviour_mut()
                    .gossipsub
                    .remove_explicit_peer(&peer_id);
                println!("❌⛓️ Connection Closed to: {peer_id} caused by {cause:?}")
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                log::debug!("❌📞 Can't dial a browser (yet) {error:?}")
            }
            event => eprintln!("🌟 Event: {event:?}\n"),
        }
    }
}

/// for a multiaddr that ends with a peer id, this strips this suffix. Rust-libp2p
/// only supports dialing to an address without providing the peer id.
fn strip_peer_id(addr: &mut Multiaddr) {
    let last = addr.pop();
    match last {
        Some(Protocol::P2p(peer_id)) => {
            let mut addr = Multiaddr::empty();
            addr.push(Protocol::P2p(peer_id));
            println!("removing peer id {addr} so this address can be dialed by rust-libp2p");
        }
        Some(other) => addr.push(other),
        _ => {}
    }
}

// Make unique messages to reply with
fn make_msg(str: &str) -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    now.to_string() + " Subscriber " + str
}
