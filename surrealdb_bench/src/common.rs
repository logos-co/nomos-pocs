use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub hash: String,
    pub parent: String,
    pub height: u64,
}

#[derive(Parser)]
pub struct Args {
    #[arg(long, default_value_t = 1_000_000)]
    pub blocks_count: usize,
}