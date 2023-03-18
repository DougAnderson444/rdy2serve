// Copyright (C) 2023 Doug Anderson
// SPDX-License-Identifier: MIT

use anyhow::Result;
use bytes::Bytes;
use clap::Parser;
use futures::StreamExt;
use libp2p::{
    core::{muxing::StreamMuxerBox, ConnectedPoint},
    floodsub::{self, Floodsub, FloodsubEvent},
    identity,
    multiaddr::{Multiaddr, Protocol},
    ping,
    swarm::{keep_alive, NetworkBehaviour, Swarm, SwarmEvent},
    webrtc, Transport,
};
use rand::thread_rng;
use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::sync::{Arc, Mutex};
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

    let mut swarm = create_swarm()?;

    // Listen for connections on the given port.
    let address = Multiaddr::from(Ipv6Addr::UNSPECIFIED)
        .with(Protocol::Udp(cli.port))
        .with(Protocol::WebRTC);

    swarm.listen_on(address.clone())?;

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
                peer_id: _,
                endpoint: ConnectedPoint::Listener { send_back_addr, .. },
                established_in,
                ..
            } => {
                eprintln!("✔️  Connection Established in {established_in:?} on {send_back_addr}")
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
                eprintln!("🏓 Ponged by {id}")
            }
            SwarmEvent::Behaviour(OutEvent::Floodsub(FloodsubEvent::Message(message))) => {
                println!(
                    "Received: '{:?}' from {:?}",
                    String::from_utf8_lossy(&message.data),
                    message.source
                );
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
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("❌⛓️ Connection Closed to: {peer_id}");

                swarm
                    .behaviour_mut()
                    .floodsub
                    .remove_node_from_partial_view(&peer_id);
            }
            event => eprintln!("🌟 Event: {event:?}\n"),
        }
    }
}

fn create_swarm() -> Result<Swarm<SuperChatBehaviour>> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = id_keys.public().to_peer_id();
    let transport = webrtc::tokio::Transport::new(
        id_keys,
        webrtc::tokio::Certificate::generate(&mut thread_rng())?,
    );

    let transport = transport
        .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)))
        .boxed();

    let floodsub_topic = floodsub::Topic::new("chat");

    let mut behaviour = SuperChatBehaviour {
        floodsub: Floodsub::new(peer_id),
        ping: ping::Behaviour::default(),
        keep_alive: keep_alive::Behaviour::default(),
    };

    behaviour.floodsub.subscribe(floodsub_topic);
    Ok(Swarm::with_tokio_executor(transport, behaviour, peer_id))
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent", prelude = "libp2p::swarm::derive_prelude")]
struct SuperChatBehaviour {
    ping: ping::Behaviour,
    floodsub: Floodsub,
    keep_alive: keep_alive::Behaviour,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum OutEvent {
    Ping(ping::Event),
    Floodsub(FloodsubEvent),
}

impl From<ping::Event> for OutEvent {
    fn from(event: ping::Event) -> Self {
        Self::Ping(event)
    }
}

impl From<FloodsubEvent> for OutEvent {
    fn from(event: FloodsubEvent) -> Self {
        Self::Floodsub(event)
    }
}

impl From<Void> for OutEvent {
    fn from(event: Void) -> Self {
        void::unreachable(event)
    }
}
