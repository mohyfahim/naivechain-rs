use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    sync::{Mutex, MutexGuard, PoisonError},
    time::SystemTime,
};

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    index: usize,
    #[serde(skip_serializing)]
    previous_hash: String,
    timestamp: u64,
    data: String,
    #[serde(skip_serializing)]
    hash: String,
}

impl Block {
    fn new(index: usize, previous_hash: &str, timestamp: u64, data: &str, hash: &str) -> Block {
        Block {
            index,
            previous_hash: previous_hash.to_string(),
            timestamp,
            data: data.to_string(),
            hash: hash.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Chain {
    next_index: usize,
    chains: Vec<Block>,
}

impl Chain {
    fn new() -> Self {
        let chains: Vec<Block> = Vec::new();
        let next_index: usize = 0;
        Chain {
            next_index,
            chains,
        }
    }

    fn is_valid_new_block(new_block: &Block, previous_block: &Block) -> bool {
        if previous_block.index + 1 != new_block.index {
            println!("invalid index {}, {}", previous_block.index , new_block.index);
            return false;
        } else if previous_block.hash != new_block.previous_hash {
            println!("invalid previoushash");
            return false;
        } else if calculate_hash_from_block(new_block) != new_block.hash {
            println!("invalid hash");
            return false;
        }
        true
    }

    fn get_latest_block(&self) -> Option<&Block> {
        self.chains.last()
    }

    fn add_block(&mut self, block: Block, genesis: bool) {
        //TODO: check the block validation before pushing to chain
        if genesis || Chain::is_valid_new_block(&block, self.get_latest_block().unwrap()) {
            self.chains.push(block);
            self.next_index += 1;
        }
    }
}
fn calculate_hash(index: usize, previous_hash: &str, timestamp: u64, data: &str) -> String {
    let block_data = format!("{}{}{}{}", index, previous_hash, timestamp, data);
    let mut hasher = Sha256::new();
    hasher.update(block_data);
    let result = hasher.finalize();
    format!("{:x}", result)
}
fn calculate_hash_from_block(block: &Block) -> String {
    let block_data = format!(
        "{}{}{}{}",
        block.index, block.previous_hash, block.timestamp, block.data
    );
    let mut hasher = Sha256::new();
    hasher.update(block_data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn handle_poisoned_lock<T>(err: PoisonError<MutexGuard<T>>) -> HttpResponse {
    // Handle the poisoned lock scenario
    eprintln!("Mutex poisoned: {:?}", err);
    HttpResponse::InternalServerError().body("Internal Server Error")
}

async fn api_blocks(data: web::Data<Mutex<Chain>>) -> impl Responder {
    match data.lock() {
        Ok(blocks) => HttpResponse::Ok().json(&*blocks),
        Err(poisoned) => handle_poisoned_lock(poisoned),
    }
}
#[derive(Deserialize)]
struct MineSchema {
    data: String,
}

async fn api_mine(msg: web::Json<MineSchema>, data: web::Data<Mutex<Chain>>) -> impl Responder {
    let mut chains = data.lock().unwrap();
    let last_block = chains.get_latest_block().unwrap();
    let index = chains.next_index;
    let previous_hash = last_block.hash.clone();
    let timestamp = get_timestamp();
    println!("chain is {chains:?}");

    chains.add_block(
        Block::new(
            index,
            previous_hash.as_str(),
            timestamp,
            msg.data.as_str(),
            &calculate_hash(index, &previous_hash, timestamp, msg.data.as_str()),
        ),
        false,
    );
    println!("chain is {chains:?}");
    HttpResponse::Ok().json(&*chains)
}
async fn api_peer() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
async fn api_add_peer() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

fn get_timestamp() -> u64 {
    let epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    epoch.as_secs()
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let block: Block = Block::new(0, "prevoius", get_timestamp(), "salam", "None");
    let mut chain: Chain = Chain::new();
    chain.add_block(block, true);

    println!("here {chain:?}");
    let shared_data = web::Data::new(Mutex::new(chain));

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/blocks")
                    .app_data(shared_data.clone())
                    .route("/get", web::get().to(api_blocks))
                    .route("/mine", web::post().to(api_mine)),
            )
            .route("/peers", web::get().to(api_peer))
            .route("/addpeer", web::get().to(api_add_peer))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await

    // let hash = calculate_hash(&block);
    // println!("{block:?} and hash: {hash}");
    // println!("chain is {chain:?}");
    // println!("chain is {chain:?}");
    // let blk = chain.get_latest_block();
    // if let Some(ref block) = blk {
    //     println!("get latest: {block:?}");
    // }
}
