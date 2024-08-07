use actix::prelude::*;
use actix::{Actor, Addr, Running, StreamHandler};
use actix_web_actors::ws;
use libp2p::{
    futures::StreamExt,
    gossipsub::{self, Topic},
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Swarm,
};
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::chain::Block;

#[derive(Serialize, Deserialize, Debug)]
pub enum P2PMessage {
    QueryLatest,
    QueryAll,
    ResponseBlockchain(Vec<Block>),
}

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
pub struct P2PNetWorkBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

#[derive(Clone)]
pub struct TransmitHandlers {
    pub swarm_tx: UnboundedSender<P2PMessage>,
    pub router_tx: UnboundedSender<P2PMessage>,
}

pub async fn handle_swarm(
    mut swarm: Swarm<P2PNetWorkBehaviour>,
    topic: gossipsub::IdentTopic,
    transmit_handler: TransmitHandlers,
    mut rx: UnboundedReceiver<P2PMessage>,
) {
    log::info!("swarm task is started");
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                log::info!("Sending: {:?}", msg);
                if let Err(e) = swarm
                .behaviour_mut().gossipsub
                .publish(topic.clone(), serde_json::to_string::<P2PMessage>(&msg).unwrap()) {
                    log::error!("Publish error: {e:?}");
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(P2PNetWorkBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        log::info!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(P2PNetWorkBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        log::info!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(P2PNetWorkBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed {
                    peer_id,
                    topic: r_topic,
                })) => {
                    log::info!("peer id {peer_id} subscribed to {r_topic}");
                    if let Err(e) = swarm
                        .behaviour_mut().gossipsub
                        .publish(topic.clone(), serde_json::to_string::<P2PMessage>(&P2PMessage::QueryLatest).unwrap()) {
                            log::error!("Publish error: {e:?}");
                        }
                }
                SwarmEvent::Behaviour(P2PNetWorkBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    let msg = String::from_utf8_lossy(&message.data);
                    log::info!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        msg,
                    );
                    transmit_handler.router_tx.send(serde_json::from_str::<P2PMessage>(&msg).unwrap()).unwrap();
                    },
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Local node is listening on {address}");
                }
                _ => {}
            }
        }
    }
}

pub fn config_network(transmit_handler: TransmitHandlers, rx: UnboundedReceiver<P2PMessage>) {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .unwrap()
        .with_quic()
        .with_behaviour(|key| {
            // To content-address message, we can take the hash of message and use it as an ID.
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            // Set a custom gossipsub configuration
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(tokio::time::Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

            // build a gossipsub network behaviour
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(P2PNetWorkBehaviour { gossipsub, mdns })
        })
        .unwrap()
        .with_swarm_config(|c| c.with_idle_connection_timeout(tokio::time::Duration::from_secs(60)))
        .build();

    let topic = gossipsub::IdentTopic::new("test-net");
    // subscribes to our topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

    // Read full lines from stdin
    // let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Listen on all interfaces and whatever port the OS assigns
    swarm
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
        .unwrap();
    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .unwrap();

    actix_web::rt::spawn(handle_swarm(swarm, topic, transmit_handler, rx));
}
