use anyhow::Error;
use clap::Parser;
use rocksdb::DB;
use serde_json::{self, from_slice};
use std::time::Instant;
use surrealdb_bench::common::{Args, Block};
use surrealdb_bench::rocksdb::{setup_blocks, setup_db};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("----------------------------------------");
    println!("[BENCHMARK START] ROCKSDB SELECT ALL");

    let args = Args::parse();
    let temp_dir = TempDir::new()?;
    let db = setup_db(&temp_dir)?;

    let start_setup = Instant::now();
    setup_blocks(&db, args.blocks_count)?;

    let setup_duration = start_setup.elapsed();
    println!(
        "[BENCHMARK SETUP] {} BLOCKS TOOK {:?}",
        args.blocks_count, setup_duration
    );

    let start_query = Instant::now();

    let blocks = fetch_all_blocks(&db, args.blocks_count)?;
    assert_eq!(blocks.len(), args.blocks_count);

    let query_duration = start_query.elapsed();
    println!(
        "[BENCHMARK RESULT] SELECTING {} BLOCKS TOOK {:?}",
        args.blocks_count, query_duration
    );

    println!("----------------------------------------\n");
    Ok(())
}

fn fetch_all_blocks(db: &DB, count: usize) -> Result<Vec<Block>, Error> {
    let mut blocks = Vec::with_capacity(count);
    let iter = db.iterator(rocksdb::IteratorMode::Start);
    for item in iter {
        let (_, value) = item?;
        let block: Block = from_slice(&value)?;
        blocks.push(block);
    }
    Ok(blocks)
}
