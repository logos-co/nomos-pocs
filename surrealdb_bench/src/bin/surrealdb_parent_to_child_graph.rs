use anyhow::Error;
use clap::Parser;
use serde::Deserialize;
use std::time::Instant;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb_bench::common::{Args, Block};
use surrealdb_bench::surrealdb::{Relation, insert_blocks, insert_edges, setup_db};
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
struct ParentBlock {
    #[serde(default)]
    parent: Vec<Block>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("----------------------------------------");
    println!("[BENCHMARK START] SURREALDB PARENT-TO-CHILD GRAPH");

    let args = Args::parse();
    let temp_dir = TempDir::new()?;
    let db = setup_db(&temp_dir).await?;

    let setup_duration = Instant::now();
    let blocks = insert_blocks(&db, args.blocks_count).await?;
    insert_edges(&db, blocks, Relation::Parent).await?;

    let setup_duration = setup_duration.elapsed();
    println!(
        "[BENCHMARK SETUP] {} BLOCKS (INCLUDING EDGES) TOOK {:?}",
        args.blocks_count, setup_duration
    );

    let start_query = Instant::now();
    let parent_blocks = fetch_parent_blocks(&db).await?;
    let parent_blocks: Vec<Block> = parent_blocks.into_iter().flat_map(|pb| pb.parent).collect();

    assert_eq!(parent_blocks.len(), args.blocks_count - 1);

    let query_duration = start_query.elapsed();
    println!(
        "[BENCHMARK RESULT] PARENT-TO-CHILD TRAVERSAL ({} BLOCKS) TOOK {:?}",
        args.blocks_count, query_duration
    );

    println!("----------------------------------------\n");
    Ok(())
}

/// Fetches parent blocks from the database by following the parent relation.
async fn fetch_parent_blocks(db: &Surreal<Db>) -> Result<Vec<ParentBlock>, Error> {
    let query = "SELECT ->parent->block.* AS parent FROM block";
    let mut response = db.query(query).await?.check()?;
    let blocks: Vec<ParentBlock> = response.take(0)?;
    Ok(blocks)
}
