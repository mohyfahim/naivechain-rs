use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub index: usize,
    #[serde(skip_serializing)]
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: String,
    #[serde(skip_serializing)]
    pub hash: String,
}

impl Block {
    pub fn new(index: usize, previous_hash: &str, timestamp: u64, data: &str, hash: &str) -> Block {
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
pub struct Chain {
    pub next_index: usize,
    pub chains: Vec<Block>,
}

impl Chain {
    pub fn new() -> Self {
        let chains: Vec<Block> = Vec::new();
        let next_index: usize = 0;
        Chain { next_index, chains }
    }

    pub fn is_valid_new_block(new_block: &Block, previous_block: &Block) -> bool {
        if previous_block.index + 1 != new_block.index {
            println!(
                "invalid index {}, {}",
                previous_block.index, new_block.index
            );
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

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chains.last()
    }

    pub fn add_block(&mut self, block: Block, genesis: bool) {
        //TODO: check the block validation before pushing to chain
        if genesis || Chain::is_valid_new_block(&block, self.get_latest_block().unwrap()) {
            self.chains.push(block);
            self.next_index += 1;
        }
    }
}
pub fn calculate_hash(index: usize, previous_hash: &str, timestamp: u64, data: &str) -> String {
    let block_data = format!("{}{}{}{}", index, previous_hash, timestamp, data);
    let mut hasher = Sha256::new();
    hasher.update(block_data);
    let result = hasher.finalize();
    format!("{:x}", result)
}
pub fn calculate_hash_from_block(block: &Block) -> String {
    let block_data = format!(
        "{}{}{}{}",
        block.index, block.previous_hash, block.timestamp, block.data
    );
    let mut hasher = Sha256::new();
    hasher.update(block_data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn get_timestamp() -> u64 {
    let epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    epoch.as_secs()
}
