use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::{
    chain::{calculate_hash, get_timestamp, Block, Chain},
    net::{P2PMessage, TransmitHandlers},
};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use libp2p::Swarm;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

pub struct ApiState {
    pub chains: Arc<Mutex<Chain>>,
    pub transmit_handlers: TransmitHandlers,
}

impl ApiState {
    pub fn new(chains: Arc<Mutex<Chain>>, transmit_handlers: TransmitHandlers) -> Self {
        Self {
            chains,
            transmit_handlers,
        }
    }
}
fn handle_poisoned_lock<T>(err: PoisonError<MutexGuard<T>>) -> HttpResponse {
    // Handle the poisoned lock scenario
    eprintln!("Mutex poisoned: {:?}", err);
    HttpResponse::InternalServerError().body("Internal Server Error")
}

#[derive(Serialize, Deserialize)]
struct GetBlocksSchema {
    index: usize,
    timestamp: u64,
    data: String,
}
impl From<&Block> for GetBlocksSchema {
    fn from(value: &Block) -> Self {
        GetBlocksSchema {
            index: value.index,
            timestamp: value.timestamp,
            data: value.data.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GetChainSchema {
    next_index: usize,
    chains: Vec<GetBlocksSchema>,
}

impl From<&Chain> for GetChainSchema {
    fn from(value: &Chain) -> Self {
        let chains: Vec<GetBlocksSchema> = value
            .chains
            .iter()
            .map(|b| Into::<GetBlocksSchema>::into(b))
            .collect();
        GetChainSchema {
            next_index: value.next_index,
            chains,
        }
    }
}

pub async fn api_blocks(state: web::Data<ApiState>) -> impl Responder {
    match state.chains.lock() {
        Ok(blocks) => HttpResponse::Ok().json(GetChainSchema::from(&*blocks)),
        Err(poisoned) => handle_poisoned_lock(poisoned),
    }
}

#[derive(Deserialize)]
pub struct MineSchema {
    data: String,
}

pub async fn api_mine(msg: web::Json<MineSchema>, data: web::Data<ApiState>) -> impl Responder {
    let mut chains = data.chains.lock().unwrap();
    let last_block = chains.get_latest_block().unwrap();
    let index = chains.next_index;
    let previous_hash = last_block.hash.clone();
    let timestamp = get_timestamp();
    println!("chain is {chains:?}");
    let new_block = Block::new(index, &previous_hash, timestamp, msg.data.as_str());
    chains.add_block(new_block.clone(), false);
    println!("chain is {chains:?}");
    if let Err(e) = data
        .transmit_handlers
        .swarm_tx
        .send(P2PMessage::ResponseBlockchain(vec![new_block]))
    {
        log::error!("error is {e}");
    }

    HttpResponse::Ok().body("ok")
}
pub async fn api_peer() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
pub async fn api_add_peer() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
