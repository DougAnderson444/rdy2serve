use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use libp2p::{
    core::muxing::StreamMuxerBox,
    identity,
    multiaddr::{Multiaddr, Protocol},
    ping,
    swarm::{keep_alive, NetworkBehaviour, Swarm},
    webrtc, Transport,
};
use rand::thread_rng;
use std::net::Ipv6Addr;
use void::Void;

#[derive(Debug, Parser)]
struct Cli {
    /// Listen for connection on this port.
    #[clap(long, default_value_t = 42069)]
    port: u16,
}

/// An example WebRTC server that will accept connections and run the ping protocol on them.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut swarm = create_swarm()?;

    // Listen for connections on the given port.
    let address = Multiaddr::from(Ipv6Addr::UNSPECIFIED)
        .with(Protocol::Udp(cli.port))
        .with(Protocol::WebRTC);

    swarm.listen_on(address)?;

    loop {
        let event = swarm.next().await.unwrap();
        eprintln!("\nNew event: {event:?}")
    }
}

fn create_swarm() -> Result<Swarm<Behaviour>> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = id_keys.public().to_peer_id();
    let transport = webrtc::tokio::Transport::new(
        id_keys,
        webrtc::tokio::Certificate::generate(&mut thread_rng())?,
    );

    let transport = transport
        .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)))
        .boxed();

    Ok(Swarm::with_tokio_executor(
        transport,
        Behaviour::default(),
        peer_id,
    ))
}

#[derive(NetworkBehaviour, Default)]
#[behaviour(
    out_event = "Event",
    event_process = false,
    prelude = "libp2p_swarm::derive_prelude"
)]
struct Behaviour {
    ping: ping::Behaviour,
    keep_alive: keep_alive::Behaviour,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum Event {
    Ping(ping::Event),
}

impl From<ping::Event> for Event {
    fn from(e: ping::Event) -> Self {
        Event::Ping(e)
    }
}

impl From<Void> for Event {
    fn from(event: Void) -> Self {
        void::unreachable(event)
    }
}
