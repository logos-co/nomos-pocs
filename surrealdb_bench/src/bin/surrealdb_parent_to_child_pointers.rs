use anyhow::Error;
use clap::Parser;
use std::time::Instant;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb_bench::common::{Args, Block};
use surrealdb_bench::surrealdb::{insert_blocks, setup_db};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("----------------------------------------");
    println!("[BENCHMARK START] SURREALDB PARENT-TO-CHILD POINTER");

    let args = Args::parse();
    let temp_dir = TempDir::new()?;
    let db = setup_db(&temp_dir).await?;

    let start_setup = Instant::now();
    insert_blocks(&db, args.blocks_count).await?;

    let setup_duration = start_setup.elapsed();
    println!(
        "[BENCHMARK SETUP] {} BLOCKS (WITHOUT EDGES) TOOK {:?}",
        args.blocks_count, setup_duration
    );

    let start_query = Instant::now();

    let target_hash = format!("block{}", args.blocks_count - 1);

    let ancestors = get_ancestors(&db, &target_hash, args.blocks_count).await?;
    assert_eq!(ancestors.len(), args.blocks_count);

    let query_duration = start_query.elapsed();
    println!(
        "[BENCHMARK RESULT] PARENT-TO-CHILD TRAVERSAL ({} BLOCKS) TOOK {:?}",
        args.blocks_count, query_duration
    );

    println!("----------------------------------------\n");
    Ok(())
}

/// Select a specific block by its hash from the database.
async fn get_block(db: &Surreal<Db>, hash: &str) -> Result<Option<Block>, Error> {
    db.select(("block", hash)).await.map_err(Into::into)
}

/// Retrieve all blocks by following the parent pointers from a given block hash.
async fn get_ancestors(db: &Surreal<Db>, hash: &str, count: usize) -> Result<Vec<Block>, Error> {
    let mut ancestors = Vec::with_capacity(count);
    let mut current_hash = hash.to_string();

    while !current_hash.is_empty() {
        if let Some(block) = get_block(db, &current_hash).await? {
            ancestors.push(block.clone());
            current_hash = block.parent;
        } else {
            break;
        }
    }
    Ok(ancestors)
}
