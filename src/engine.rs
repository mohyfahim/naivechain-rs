use actix_web::web;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    net::{P2PMessage, TransmitHandlers},
    ApiState,
};

pub async fn handle_engine(
    shared_states: web::Data<ApiState>,
    handlers: TransmitHandlers,
    mut rx: UnboundedReceiver<P2PMessage>,
) {
    let receiver_handler = |msg| -> () {
        log::info!("Sending: {:?}", msg);
        match msg {
            P2PMessage::QueryAll => {
                let chain = shared_states.chains.lock().unwrap().chains.to_vec();
                handlers
                    .swarm_tx
                    .send(P2PMessage::ResponseBlockchain(chain))
                    .unwrap();
            }
            P2PMessage::QueryLatest => {
                let latest_block = shared_states
                    .chains
                    .lock()
                    .unwrap()
                    .get_latest_block()
                    .unwrap()
                    .clone();
                handlers
                    .swarm_tx
                    .send(P2PMessage::ResponseBlockchain(vec![latest_block]))
                    .unwrap();
            }
            P2PMessage::ResponseBlockchain(received_chain) => {
                let mut chains = shared_states.chains.lock().unwrap();
                let latest_block_held = chains.get_latest_block().unwrap();
                let lastes_block_received = received_chain.last().unwrap();
                if lastes_block_received.index > latest_block_held.index {
                    log::info!(
                        "blockchain possibly behind. We got:   {} + ' Peer got: ' + {}",
                        latest_block_held.index,
                        lastes_block_received.index
                    );
                    if latest_block_held.hash == lastes_block_received.previous_hash {
                        log::info!("We can append the received block to our chain");
                        chains.add_block(lastes_block_received.clone());
                        let latest_block = chains.get_latest_block().unwrap();
                        handlers
                            .swarm_tx
                            .send(P2PMessage::ResponseBlockchain(vec![latest_block.clone()]))
                            .unwrap();
                    } else if received_chain.len() == 1 {
                        log::info!("We have to query the chain from our peer");
                        handlers.swarm_tx.send(P2PMessage::QueryAll).unwrap();
                    } else {
                        log::info!("Received blockchain is longer than current blockchain");
                        if chains.replace_block_chain(received_chain.clone()) {
                            handlers
                                .swarm_tx
                                .send(P2PMessage::ResponseBlockchain(received_chain))
                                .unwrap();
                        }
                    }
                } else {
                    log::info!(
                        "received blockchain is not longer than current blockchain. Do nothing"
                    );
                }
            }
            P2PMessage::QueryPeers => {
                log::info!("query peers");
                handlers.swarm_tx.send(P2PMessage::QueryPeers).unwrap();
            }
            P2PMessage::ResponsePeers(peers) => {
                log::info!("response peers peers");
                handlers
                    .api_peers_tx
                    .send(P2PMessage::ResponsePeers(peers))
                    .unwrap();
            }
            P2PMessage::AddPeer(_) => {
                log::info!("add peer");
                handlers.swarm_tx.send(msg).unwrap();
            },
        }
    };
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                receiver_handler(msg);
            }
        }
    }
}
