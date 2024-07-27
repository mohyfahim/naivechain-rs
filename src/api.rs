use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use serde::Deserialize;

use crate::{
    chain::{calculate_hash, get_timestamp, Block, Chain},
    net::{P2PClientsType, P2PWebSocket},
};

pub struct ApiState {
    pub chains: Arc<Mutex<Chain>>,
    pub clients: P2PClientsType,
}

impl ApiState {
    pub fn new(chains: Arc<Mutex<Chain>>) -> Self {
        let clients:  P2PClientsType = Arc::new(Mutex::new(Vec::new()));
        Self { chains, clients }
    }
}
fn handle_poisoned_lock<T>(err: PoisonError<MutexGuard<T>>) -> HttpResponse {
    // Handle the poisoned lock scenario
    eprintln!("Mutex poisoned: {:?}", err);
    HttpResponse::InternalServerError().body("Internal Server Error")
}

pub async fn api_blocks(state: web::Data<Arc<Mutex<ApiState>>>) -> impl Responder {
    match state.lock().unwrap().chains.lock() {
        Ok(blocks) => HttpResponse::Ok().json(&*blocks),
        Err(poisoned) => handle_poisoned_lock(poisoned),
    }
}
#[derive(Deserialize)]
pub struct MineSchema {
    data: String,
}

pub async fn api_mine(msg: web::Json<MineSchema>, data: web::Data<Arc<Mutex<ApiState>>>) -> impl Responder {
    let ld = data.lock().unwrap();
    let mut chains = ld.chains.lock().unwrap();
    let last_block = chains.get_latest_block().unwrap();
    let index = chains.next_index;
    let previous_hash = last_block.hash.clone();
    let timestamp = get_timestamp();
    println!("chain is {chains:?}");

    chains.add_block(
        Block::new(
            index,
            &previous_hash,
            timestamp,
            msg.data.as_str(),
            &calculate_hash(index, &previous_hash, timestamp, msg.data.as_str()),
        ),
        false,
    );
    println!("chain is {chains:?}");
    HttpResponse::Ok().body("ok")
}
pub async fn api_peer() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
pub async fn api_add_peer() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

pub async fn api_start_ws(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<Arc<Mutex<ApiState>>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        P2PWebSocket::new(state.lock().unwrap().clients.clone()),
        &req,
        stream,
    )
}
