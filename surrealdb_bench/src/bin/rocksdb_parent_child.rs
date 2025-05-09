use anyhow::Error;
use clap::Parser;
use rocksdb::DB;
use serde_json::{self, from_slice};
use std::time::Instant;
use surrealdb_bench::common::{Args, Block};
use surrealdb_bench::rocksdb::{setup_blocks, setup_db};
use tempfile::TempDir;

fn main() -> Result<(), Error> {
    println!("----------------------------------------");
    println!("[BENCHMARK START] ROCKSDB PARENT-TO-CHILD");

    let args = Args::parse();
    let temp_dir = TempDir::new()?;
    let db = setup_db(&temp_dir)?;

    let start_setup = Instant::now();
    setup_blocks(&db, args.blocks_count)?;

    let duration_setup = start_setup.elapsed();
    println!(
        "[BENCHMARK SETUP] {} BLOCKS TOOK {:?}",
        args.blocks_count, duration_setup
    );

    let start = Instant::now();
    let target_hash = format!("block{}", args.blocks_count - 1);

    let ancestors = get_ancestors(&db, &target_hash, args.blocks_count)?;
    assert_eq!(ancestors.len(), args.blocks_count);

    let duration = start.elapsed();
    println!(
        "[BENCHMARK RESULT] PARENT-TO-CHILD TRAVERSAL ({} BLOCKS) TOOK: {:?}",
        args.blocks_count, duration
    );

    println!("----------------------------------------\n");
    Ok(())
}

/// Fetch a block from the database by its hash.
fn get_block(db: &DB, hash: &str) -> Result<Option<Block>, Error> {
    let key = hash.as_bytes();
    Ok(match db.get(key)? {
        Some(value) => Some(from_slice(&value)?),
        None => None,
    })
}

/// Retrieve all ancestors of a block by following the parent pointers.
fn get_ancestors(db: &DB, hash: &str, count: usize) -> Result<Vec<Block>, Error> {
    let mut ancestors = Vec::with_capacity(count);
    let mut current_hash = hash.to_string();

    while !current_hash.is_empty() {
        if let Some(block) = get_block(db, &current_hash)? {
            current_hash = block.parent.clone();
            ancestors.push(block);
        } else {
            break;
        }
    }
    Ok(ancestors)
}
