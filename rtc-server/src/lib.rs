// Copyright (C) 2023 Doug Anderson
// SPDX-License-Identifier: MIT

use anyhow::Result;
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
use std::net::Ipv6Addr;
use tokio::sync::oneshot;
use void::Void;

#[derive(Debug, Parser)]
struct Cli {
    /// Listen for connection on this port.
    #[clap(long, default_value_t = 42069)]
    port: u16,
}

/// An example WebRTC server that will accept connections and run the ping protocol on them.
pub async fn start(sender: oneshot::Sender<String>) -> Result<()> {
    let mut sender = Some(sender);
    let cli = Cli::parse();

    let mut swarm = create_swarm()?;

    // Listen for connections on the given port.
    let address = Multiaddr::from(Ipv6Addr::UNSPECIFIED)
        .with(Protocol::Udp(cli.port))
        .with(Protocol::WebRTC);

    swarm.listen_on(address.clone())?;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr {
                address,
                listener_id,
            } => {
                // check if address string contains "::" at all, if so skip the connection prompt
                if !address.to_string().contains("::") {
                    // add p2p PeerId to address as p2p Protocol
                    let full_address = address
                        .with(Protocol::P2p(*swarm.local_peer_id().as_ref()))
                        .to_string();

                    // take the frst address assigned by the os
                    if let Some(sender) = sender.take() {
                        sender.send(full_address.clone()).unwrap();
                    }
                } else {
                    println!("\nListening on {address}\n {listener_id:?}");
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
