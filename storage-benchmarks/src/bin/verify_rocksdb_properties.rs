use nomos_storage::backends::{rocksdb::RocksBackend, StorageBackend as _};
use storage_benchmarks::BenchConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = BenchConfig::production();

    if !config.settings.db_path.exists() {
        println!(
            "No database found at: {}",
            config.settings.db_path.display()
        );
        return Err("Database not found - run dataset_generator first".into());
    }

    let storage = RocksBackend::new(config.settings)?;

    println!("=== RocksDB Properties Verification ===");
    println!();

    verify_property_availability(&storage);

    Ok(())
}

fn verify_property_availability(storage: &RocksBackend) {
    let level0_prop = rocksdb::properties::num_files_at_level(0);
    let level1_prop = rocksdb::properties::num_files_at_level(1);
    let level2_prop = rocksdb::properties::num_files_at_level(2);

    let properties_to_test = vec![
        ("STATS", rocksdb::properties::STATS),
        (
            "BLOCK_CACHE_CAPACITY",
            rocksdb::properties::BLOCK_CACHE_CAPACITY,
        ),
        (
            "TOTAL_SST_FILES_SIZE",
            rocksdb::properties::TOTAL_SST_FILES_SIZE,
        ),
        (
            "CUR_SIZE_ALL_MEM_TABLES",
            rocksdb::properties::CUR_SIZE_ALL_MEM_TABLES,
        ),
        (
            "LIVE_SST_FILES_SIZE",
            rocksdb::properties::LIVE_SST_FILES_SIZE,
        ),
        ("ESTIMATE_NUM_KEYS", rocksdb::properties::ESTIMATE_NUM_KEYS),
        ("NUM_FILES_AT_LEVEL0", &level0_prop),
        ("NUM_FILES_AT_LEVEL1", &level1_prop),
        ("NUM_FILES_AT_LEVEL2", &level2_prop),
    ];

    let custom_properties = vec![
        "rocksdb.index-and-filter-cache.usage",
        "rocksdb.index-and-filter-cache.capacity",
        "rocksdb.compaction-pending",
        "rocksdb.number.compactions",
        "rocksdb.compact.read.bytes",
        "rocksdb.compact.write.bytes",
        "rocksdb.compaction.cpu.time",
        "rocksdb.mem-table-flush-pending",
        "rocksdb.space.amplification",
        "rocksdb.total-sst-files-size",
        "rocksdb.number.keys.deleted",
        "rocksdb.size-bytes-at-level0",
        "rocksdb.size-bytes-at-level1",
    ];

    println!("Standard RocksDB Properties:");
    for (name, prop) in properties_to_test {
        test_standard_property(storage, name, &prop.to_string());
    }

    println!("\nCustom/Extended Properties:");
    for prop_name in custom_properties {
        test_custom_property(storage, prop_name);
    }

    println!("\nSTATS Property Sample:");
    test_stats_property(storage);
}

fn test_standard_property(storage: &RocksBackend, name: &str, property: &str) {
    let property_owned = property.to_string();
    let transaction = storage.txn(move |db| match db.property_value(&property_owned) {
        Ok(Some(value)) => Ok(Some(value.into_bytes().into())),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    });

    match transaction.execute() {
        Ok(Some(value_bytes)) => {
            let value_str = String::from_utf8_lossy(&value_bytes);
            let truncated = if value_str.len() > 100 {
                format!("{}...", &value_str[..100])
            } else {
                value_str.to_string()
            };
            println!("OK {}: {}", name, truncated);
        }
        Ok(None) => {
            println!("FAIL {}: None (property exists but no value)", name);
        }
        Err(e) => {
            println!("FAIL {}: Error - {}", name, e);
        }
    }
}

fn test_custom_property(storage: &RocksBackend, property: &str) {
    let prop_owned = property.to_string();
    let transaction = storage.txn(move |db| match db.property_value(&prop_owned) {
        Ok(Some(value)) => Ok(Some(value.into_bytes().into())),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    });

    match transaction.execute() {
        Ok(Some(value_bytes)) => {
            let value_str = String::from_utf8_lossy(&value_bytes);
            println!("OK {}: {}", property, value_str.trim());
        }
        Ok(None) => {
            println!("FAIL {}: None (property exists but no value)", property);
        }
        Err(e) => {
            println!("FAIL {}: Error - {}", property, e);
        }
    }
}

fn test_stats_property(storage: &RocksBackend) {
    let transaction = storage.txn(|db| match db.property_value(rocksdb::properties::STATS) {
        Ok(Some(stats)) => Ok(Some(stats.into_bytes().into())),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    });

    match transaction.execute() {
        Ok(Some(stats_bytes)) => {
            let stats_str = String::from_utf8_lossy(&stats_bytes);
            println!("Sample STATS lines:");
            for (i, line) in stats_str.lines().take(10).enumerate() {
                println!("  {}: {}", i + 1, line);
            }
            if stats_str.lines().count() > 10 {
                println!("  ... ({} total lines)", stats_str.lines().count());
            }
        }
        Ok(None) => {
            println!("FAIL STATS: None");
        }
        Err(e) => {
            println!("FAIL STATS: Error - {}", e);
        }
    }
}
