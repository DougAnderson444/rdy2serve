use libp2p::{
    gossipsub, identity, ping,
    swarm::{keep_alive, NetworkBehaviour},
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use void::Void;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent", prelude = "libp2p::swarm::derive_prelude")]
pub struct SuperChatBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    ping: ping::Behaviour,
    keep_alive: keep_alive::Behaviour,
}

impl SuperChatBehaviour {
    pub fn new(id_keys: identity::Keypair) -> Self {
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            // .mesh_n_low(2) // experiment to see if this matters for WebRTC
            // .support_floodsub()
            .check_explicit_peers_ticks(1)
            .heartbeat_initial_delay(Duration::from_secs(30))
            .heartbeat_interval(Duration::from_secs(60)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
            .build()
            .expect("Valid config");

        Self {
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(id_keys),
                gossipsub_config,
            )
            .expect("Valid configuration"),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(30))),
            keep_alive: keep_alive::Behaviour,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum OutEvent {
    Gossipsub(gossipsub::Event),
    Ping(ping::Event),
}

impl From<gossipsub::Event> for OutEvent {
    fn from(event: gossipsub::Event) -> Self {
        OutEvent::Gossipsub(event)
    }
}

impl From<ping::Event> for OutEvent {
    fn from(event: ping::Event) -> Self {
        OutEvent::Ping(event)
    }
}

impl From<Void> for OutEvent {
    fn from(event: Void) -> Self {
        void::unreachable(event)
    }
}
