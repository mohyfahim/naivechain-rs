use actix_web::{
    web::{self},
    App, HttpServer,
};
use api::*;
use chain::{Block, Chain};
use clap::Parser;
use net::{P2PMessage, ReceiveHandlers, TransmitHandlers};
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::unbounded_channel;
mod api;
mod chain;
mod engine;
mod net;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short, long, value_delimiter = ',', num_args = 1..)]
    pub list: Option<Vec<String>>,
    #[clap(short, long)]
    pub port: u16,
}

#[actix_web::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let cli = Cli::parse();
    let (tx, rx) = unbounded_channel::<P2PMessage>();
    let (tx_router, rx_router) = unbounded_channel::<P2PMessage>();
    let (tx_api_peers, rx_api_peers) = unbounded_channel::<P2PMessage>();

    let transmit_handlers = TransmitHandlers {
        swarm_tx: tx.clone(),
        router_tx: tx_router.clone(),
        api_peers_tx: tx_api_peers.clone(),
    };

    let receive_handlers = ReceiveHandlers {
        api_peers_rx: Mutex::new(rx_api_peers),
    };

    let genesis_block: Block = Chain::get_genesis_block();
    let chain: Chain = Chain::new(genesis_block);

    println!("here {chain:?}");
    let api_states: ApiState =
        ApiState::new(Arc::new(Mutex::new(chain)), transmit_handlers.clone(), receive_handlers);
    let shared_states = web::Data::new(api_states);

    net::config_network(transmit_handlers.clone(), rx);
    actix_web::rt::spawn(engine::handle_engine(
        shared_states.clone(),
        transmit_handlers.clone(),
        rx_router,
    ));

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(shared_states.clone())
            .service(
                web::scope("/blocks")
                    .route("/get", web::get().to(api_blocks))
                    .route("/mine", web::post().to(api_mine)),
            )
            .route("/peers", web::get().to(api_peer))
            .route("/addpeer", web::post().to(api_add_peer))
    })
    .bind(("127.0.0.1", cli.port))
    .unwrap()
    .run()
    .await;
}
