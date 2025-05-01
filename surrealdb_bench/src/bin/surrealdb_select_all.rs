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
    println!("[BENCHMARK START] SURREALDB SELECT ALL");

    let args = Args::parse();
    let temp_dir = TempDir::new()?;
    let db = setup_db(&temp_dir).await?;

    let start_setup = Instant::now();
    insert_blocks(&db, args.blocks_count).await?;

    let start_setup = start_setup.elapsed();
    println!(
        "[BENCHMARK SETUP] {} BLOCKS TOOK {:?}",
        args.blocks_count, start_setup
    );

    let start_query = Instant::now();

    let blocks = fetch_all_blocks(&db).await?;
    assert_eq!(blocks.len(), args.blocks_count,);

    let query_duration = start_query.elapsed();
    println!(
        "[BENCHMARK RESULT] SELECTING {} BLOCKS TOOK {:?}",
        args.blocks_count, query_duration
    );

    println!("----------------------------------------\n");
    Ok(())
}

/// Fetches all blocks from the database.
async fn fetch_all_blocks(db: &Surreal<Db>) -> Result<Vec<Block>, Error> {
    let query = "SELECT * FROM block";
    let mut response = db.query(query).await?;
    let blocks: Vec<Block> = response.take(0)?;
    Ok(blocks)
}
