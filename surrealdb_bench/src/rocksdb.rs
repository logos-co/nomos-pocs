use crate::surrealdb::create_n_blocks;
use anyhow::Error;
use rocksdb::DB;
use serde_json::to_vec;
use tempfile::TempDir;

pub fn setup_db(temp_dir: &TempDir) -> Result<DB, Error> {
    Ok(DB::open_default(temp_dir.path())?)
}

pub fn setup_blocks(db: &DB, count: usize) -> Result<(), Error> {
    let blocks = create_n_blocks(count);
    let mut batch = rocksdb::WriteBatch::default();
    for block in &blocks {
        let key = block.hash.as_bytes();
        let value = to_vec(block)?;
        batch.put(key, value);
    }
    db.write(batch)?;
    Ok(())
}
