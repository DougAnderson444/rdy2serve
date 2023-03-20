// Copyright (C) 2023 Doug Anderson
// SPDX-License-Identifier: MIT

use anyhow::Result;
use bytes::Bytes;
use clap::Parser;
use futures::StreamExt;
use libp2p::{
    core::{muxing::StreamMuxerBox, ConnectedPoint},
    gossipsub, identity,
    multiaddr::{Multiaddr, Protocol},
    ping::{self, Config},
    swarm::{
        dial_opts::{DialOpts, PeerCondition},
        keep_alive, NetworkBehaviour, SwarmBuilder, SwarmEvent,
    },
    webrtc, PeerId, Transport,
};
use rand::thread_rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::Ipv6Addr;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, time::Duration};
use tokio::sync::{mpsc, oneshot};
use void::Void;

#[derive(Debug, Parser)]
struct Cli {
    /// Listen for connection on this port.
    #[clap(long, default_value_t = 42069)]
    port: u16,
}

#[derive(Debug, Clone)]
pub struct ServerResponse {
    pub address: Bytes,
}

pub struct Message<T> {
    pub reply: Responder<T>,
}

type Responder<T> = oneshot::Sender<T>;

/// An example WebRTC server that will accept connections and run the ping protocol on them.
pub async fn start(mut request_recvr: mpsc::Receiver<Message<ServerResponse>>) -> Result<()> {
    // let mut config: Config = Config::default();
    let config: Arc<Mutex<HashMap<String, Bytes>>> = Arc::new(Mutex::new(HashMap::new()));
    let config_getter = Arc::clone(&config);

    // Spawn an API manager to receive incoming Requests
    tokio::spawn(async move {
        log::debug!(">>>> Reply listener spawned.");
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

    let cli = Cli::parse();

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

        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(12)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            // .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
            // one of the 10 second timeouts is killing the Connection
            // .unsubscribe_backoff(30)
            // .graft_flood_threshold(Duration::from_secs(30))
            // .published_message_ids_cache_time(Duration::from_secs(30))
            .build()
            .expect("Valid config");

        let mut behaviour = SuperChatBehaviour {
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(id_keys),
                gossipsub_config,
            )
            .expect("Valid configuration"),
            ping: ping::Behaviour::new(Config::new().with_interval(Duration::new(1, 0))),
            keep_alive: keep_alive::Behaviour::default(),
        };

        behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();

        SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build()
    };

    // Listen for connections on the given port.
    let address = Multiaddr::from(Ipv6Addr::UNSPECIFIED)
        .with(Protocol::Udp(cli.port))
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
                let dial_opts = DialOpts::peer_id(PeerId::random())
                    .condition(PeerCondition::NotDialing)
                    .addresses(vec![send_back_addr.clone()])
                    .extend_addresses_through_behaviour()
                    .build();
                if let Err(e) = swarm.dial(dial_opts) {
                    println!("Dialing error: {e:?}");
                }
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id)
            }
            SwarmEvent::Behaviour(OutEvent::Ping(ping::Event {
                peer,
                result: Ok(ping::Success::Ping { rtt }),
            })) => {
                // if let Err(e) = swarm
                //     .behaviour_mut()
                //     .gossipsub
                //     .publish(gossipsub_topic.clone(), "Hey there Subscriber".as_bytes())
                // {
                //     println!("Publish error: {e:?}");
                // }
                let id = peer.to_string().to_owned();
                eprintln!("🏐 Pinged {id} ({rtt:?})")
            }
            SwarmEvent::Behaviour(OutEvent::Ping(ping::Event {
                peer,
                result: Ok(ping::Success::Pong),
            })) => {
                let id = peer.to_string().to_owned();
                // swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                eprintln!("🏓 Ponged by {id}")
            }
            SwarmEvent::Behaviour(OutEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                println!(
                    "📨 Got message: '{}' with id: {id} from peer: {peer_id}",
                    String::from_utf8_lossy(&message.data),
                )
            }
            SwarmEvent::Behaviour(OutEvent::Gossipsub(gossipsub::Event::Subscribed {
                peer_id,
                topic, // : gossipsub::TopicHash { hash },
            })) => {
                println!("💨💨💨  {topic:?} Subscriber: {peer_id}");
                // swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);

                if let Err(e) = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(gossipsub_topic.clone(), "Hey there Subscriber".as_bytes())
                {
                    println!("Publish error: {e:?}");
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
                println!("❌⛓️ Connection Closed to: {peer_id} caused by {cause:?}");
            }
            event => eprintln!("🌟 Event: {event:?}\n"),
        }
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent", prelude = "libp2p::swarm::derive_prelude")]
struct SuperChatBehaviour {
    ping: ping::Behaviour,
    gossipsub: gossipsub::Behaviour,
    keep_alive: keep_alive::Behaviour,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum OutEvent {
    Ping(ping::Event),
    Gossipsub(gossipsub::Event),
}

impl From<ping::Event> for OutEvent {
    fn from(event: ping::Event) -> Self {
        Self::Ping(event)
    }
}
impl From<gossipsub::Event> for OutEvent {
    fn from(event: gossipsub::Event) -> Self {
        OutEvent::Gossipsub(event)
    }
}

impl From<Void> for OutEvent {
    fn from(event: Void) -> Self {
        void::unreachable(event)
    }
}
