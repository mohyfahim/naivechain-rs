use actix_web::{
    web::{self},
    App, HttpServer,
};
use api::*;
use chain::{Block, Chain};
use net::P2PWebSocket;
use std::sync::{Arc, Mutex};
mod api;
mod chain;
mod net;

async fn run_periodic_task(state: Arc<Mutex<ApiState>>) {
    let interval = tokio::time::Duration::from_secs(10);
    loop {
        actix::clock::sleep(interval).await;
        let ls = state.lock().unwrap();
        let clients = ls.clients.lock().unwrap();
        // for client in clients.iter() {
        //     println!("clients :{:?}", client);
        //     // client.do_send(ws::Message::Text("Periodic message".to_string()));
        // }
        P2PWebSocket::broadcast(&clients, "come on");
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let block: Block = Block::new(0, "prevoius", chain::get_timestamp(), "salam", "None");
    let mut chain: Chain = Chain::new();
    chain.add_block(block, true);

    println!("here {chain:?}");
    let api_states: Arc<Mutex<ApiState>> =
        Arc::new(Mutex::new(ApiState::new(Arc::new(Mutex::new(chain)))));

    actix_web::rt::spawn(run_periodic_task(api_states.clone()));

    let shared_states = web::Data::new(api_states);

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/blocks")
                    .app_data(shared_states.clone())
                    .route("/get", web::get().to(api_blocks))
                    .route("/mine", web::post().to(api_mine)),
            )
            .service(
                web::scope("/ws")
                    .app_data(shared_states.clone())
                    .route("/", web::get().to(api_start_ws)),
            )
            .route("/peers", web::get().to(api_peer))
            .route("/addpeer", web::get().to(api_add_peer))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
