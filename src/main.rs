use std::time::SystemTime;

#[derive(Debug)]
struct Block {
    index: u32,
    previous_hash: String,
    timestamp: u64,
    data: String,
    hash: String,
}

impl Block {
    fn new(index: u32, previous_hash: &str, timestamp: u64, data: &str, hash: &str) -> Block {
        Block {
            index,
            previous_hash: previous_hash.to_string(),
            timestamp,
            data: data.to_string(),
            hash: hash.to_string(),
        }
    }
}

fn main() {
    let epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let block: Block = Block::new(0, "prevoius", epoch.as_secs(), "salam", "None");
    println!("{block:?}");
}
