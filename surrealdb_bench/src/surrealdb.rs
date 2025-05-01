use crate::common::Block;
use anyhow::Error;
use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
pub enum Relation {
    Parent,
    Child,
}

pub async fn setup_db(temp_dir: &TempDir) -> Result<Surreal<Db>, Error> {
    let db_path = temp_dir.path().join("surreal.db");
    let db = Surreal::new::<RocksDb>(db_path).await?;
    db.use_ns("nomos").use_db("nms").await?;
    Ok(db)
}

pub async fn insert_blocks(db: &Surreal<Db>, count: usize) -> Result<Vec<Block>, Error> {
    let blocks = create_n_blocks(count);
    let queries: Vec<String> = blocks
        .iter()
        .map(|block| {
            Ok(format!(
                "CREATE block:{} CONTENT {};",
                block.hash,
                serde_json::to_string(block)?
            ))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    db.query(queries.join(" ")).await?.check()?;
    Ok(blocks)
}

pub async fn insert_edges(
    db: &Surreal<Db>,
    blocks: Vec<Block>,
    relation: Relation,
) -> Result<(), Error> {
    let blocks = match relation {
        Relation::Parent => blocks.into_iter().rev().collect::<Vec<_>>(),
        Relation::Child => blocks,
    };

    let queries: Vec<String> = blocks
        .iter()
        .filter(|block| !block.parent.is_empty())
        .map(|block| match relation {
            Relation::Parent => format!(
                "RELATE block:{}->parent->block:{}",
                block.hash, block.parent
            ),
            Relation::Child => {
                format!("RELATE block:{}->child->block:{}", block.parent, block.hash)
            }
        })
        .collect();

    if !queries.is_empty() {
        db.query(queries.join("; ")).await?.check()?;
    }
    Ok(())
}

pub fn create_n_blocks(n: usize) -> Vec<Block> {
    (0..n)
        .map(|i| Block {
            hash: format!("block{}", i),
            parent: if i == 0 {
                String::new()
            } else {
                format!("block{}", i - 1)
            },
            height: i as u64 + 1,
        })
        .collect()
}
