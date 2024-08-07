use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub index: usize,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: String,
    pub hash: String,
}

impl Block {
    pub fn _new_with_hash(
        index: usize,
        previous_hash: &str,
        timestamp: u64,
        data: &str,
        hash: &str,
    ) -> Block {
        Block {
            index,
            previous_hash: previous_hash.to_string(),
            timestamp,
            data: data.to_string(),
            hash: hash.to_string(),
        }
    }

    pub fn new(index: usize, previous_hash: &str, timestamp: u64, data: &str) -> Block {
        let hash = calculate_hash(index, previous_hash, timestamp, data);
        Block {
            index,
            previous_hash: previous_hash.to_string(),
            timestamp,
            data: data.to_string(),
            hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chain {
    pub next_index: usize,
    pub chains: Vec<Block>,
}

impl Chain {
    pub fn new(genesis_block: Block) -> Self {
        let chains: Vec<Block> = vec![genesis_block];
        let next_index: usize = 1;
        Chain { next_index, chains }
    }

    pub fn replace_block_chain(&mut self, new_blocks: Vec<Block>) -> bool {
        if Self::is_valid_chain(&new_blocks) && self.chains.len() < new_blocks.len() {
            log::info!("Received blockchain is valid. Replacing current blockchain with received blockchain");
            self.chains = new_blocks;
            return true;
        } else {
            log::error!("Received blockchain is invalid");
        }
        false
    }
    pub fn get_genesis_block() -> Block {
        Block::new(0, "0", 1723020013, "genesis")
    }
    pub fn is_valid_chain(chain: &Vec<Block>) -> bool {
        if chain.get(0).unwrap() != &Self::get_genesis_block() {
            return false;
        }
        for (b1, b2) in chain.iter().zip(chain.iter().skip(1)) {
            if !Self::is_valid_new_block(b2, b1) {
                return false;
            }
        }
        true
    }
    pub fn is_valid_new_block(new_block: &Block, previous_block: &Block) -> bool {
        log::info!("new block {:?}, prev block {:?}", new_block, previous_block);
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

    pub fn add_block(&mut self, block: Block) {
        if Chain::is_valid_new_block(&block, self.get_latest_block().unwrap()) {
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
