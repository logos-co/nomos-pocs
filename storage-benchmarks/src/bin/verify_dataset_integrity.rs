use std::time::Instant;

use log::info;
use nomos_storage::{
    api::{chain::StorageChainApi as _, da::StorageDaApi as _},
    backends::{rocksdb::RocksBackend, StorageBackend as _},
};
use storage_benchmarks::{
    benchmark::utilities::{create_blob_id, create_header_id},
    BenchConfig, CompletenessSummary, DataSizesSummary, DatasetVerificationReport,
    VerificationPerformance, VerificationSummary, WarningsSummary,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = BenchConfig::production();

    if !config.settings.db_path.exists() {
        println!(
            "No database found at: {}",
            config.settings.db_path.display()
        );
        return Err("Database not found".into());
    }

    info!("Opening database: {}", config.settings.db_path.display());

    let mut storage_settings = config.settings.clone();
    storage_settings.read_only = true;

    let mut storage = RocksBackend::new(storage_settings)?;

    info!("Starting database verification");
    info!("=== Database Verification ===");

    info!("Checking blocks...");
    let start_time = Instant::now();
    let mut blocks_found = 0;
    let mut total_block_size = 0u64;

    for chunk_start in (0..1_100_000).step_by(10_000) {
        let mut chunk_found = 0;
        let chunk_end = chunk_start + 10_000;

        for i in chunk_start..chunk_end {
            let header_id = create_header_id(i);

            match storage.get_block(header_id).await {
                Ok(Some(data)) => {
                    blocks_found += 1;
                    total_block_size += data.len() as u64;
                    chunk_found += 1;
                }
                Ok(None) => {
                    if chunk_found == 0 {
                        info!("No more blocks found after block {}", i);
                        break;
                    }
                }
                Err(_) => {}
            }
        }

        if chunk_found == 0 {
            break;
        }

        info!(
            "Blocks {}-{}: found {} blocks",
            chunk_start,
            chunk_start + chunk_found - 1,
            chunk_found
        );
    }

    let blocks_check_time = start_time.elapsed();

    println!("Block Data:");
    println!("   Blocks found: {}", blocks_found);
    println!("   Expected blocks: 1,051,200");
    println!(
        "   Total block size: {:.1} GB",
        total_block_size as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!(
        "   Average block size: {} bytes",
        if blocks_found > 0 {
            total_block_size / blocks_found
        } else {
            0
        }
    );
    println!("   Check time: {:.1}s", blocks_check_time.as_secs_f64());
    println!();

    info!("Checking DA shares...");
    let start_time = Instant::now();
    let mut shares_found = 0;
    let mut total_share_size = 0u64;
    let mut commitments_found = 0;
    let mut total_commitment_size = 0u64;

    for blob_idx in 0..1000 {
        for subnet in 0..50 {
            let blob_id = create_blob_id(blob_idx, 0);
            let share_idx = [subnet as u8, 0u8];

            if let Ok(Some(data)) = storage.get_light_share(blob_id, share_idx).await {
                shares_found += 1;
                total_share_size += data.len() as u64;
            }

            if let Ok(Some(data)) = storage.get_shared_commitments(blob_id).await {
                commitments_found += 1;
                total_commitment_size += data.len() as u64;
            }
        }

        if blob_idx % 100 == 0 {
            info!(
                "Checked blob {} - found {} shares, {} commitments so far",
                blob_idx, shares_found, commitments_found
            );
        }
    }

    let da_check_time = start_time.elapsed();

    println!("DA Data:");
    println!(
        "   DA shares found: {} (sampled from first 50K possibilities)",
        shares_found
    );
    println!("   Expected DA shares: ~256,650 total");
    println!(
        "   Total share size: {:.1} MB",
        total_share_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "   Average share size: {} bytes",
        if shares_found > 0 {
            total_share_size / shares_found
        } else {
            0
        }
    );
    println!();
    println!("   Commitments found: {}", commitments_found);
    println!(
        "   Total commitment size: {:.1} GB",
        total_commitment_size as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!(
        "   Average commitment size: {} bytes",
        if commitments_found > 0 {
            total_commitment_size / commitments_found
        } else {
            0
        }
    );
    println!("   Check time: {:.1}s", da_check_time.as_secs_f64());
    println!();

    let total_verified_size = total_block_size + total_share_size + total_commitment_size;

    println!("Summary:");
    println!("   Database on disk: 4.8 GB");
    println!(
        "   Verified data size: {:.1} GB",
        total_verified_size as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!(
        "   Blocks completeness: {:.1}%",
        blocks_found as f64 / 1_051_200.0 * 100.0
    );
    println!(
        "   Estimated DA completeness: {:.1}%",
        shares_found as f64 / (256_650.0 / 50.0) * 100.0
    );

    if blocks_found < 1_000_000 {
        println!("WARNING:  Block generation may have been incomplete");
    }

    if total_verified_size < 50 * 1024 * 1024 * 1024 {
        println!("WARNING:  Data size much smaller than expected - check generation logic");
    }

    save_verification_report(
        blocks_found as usize,
        shares_found as usize,
        commitments_found as usize,
        total_block_size,
        total_share_size,
        total_commitment_size,
        blocks_check_time + da_check_time,
    );

    Ok(())
}

fn save_verification_report(
    blocks_found: usize,
    shares_found: usize,
    commitments_found: usize,
    total_block_size: u64,
    total_share_size: u64,
    total_commitment_size: u64,
    verification_time: std::time::Duration,
) {
    let results_dir = BenchConfig::results_path();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("dataset_verification_{}.json", timestamp);
    let filepath = results_dir.join(filename);

    let total_verified_size = total_block_size + total_share_size + total_commitment_size;

    let report = DatasetVerificationReport {
        verification_summary: VerificationSummary {
            blocks_found,
            da_shares_found: shares_found,
            da_commitments_found: commitments_found,
            total_objects_found: blocks_found + shares_found + commitments_found,
        },
        data_sizes: DataSizesSummary {
            total_block_size_bytes: total_block_size,
            total_share_size_bytes: total_share_size,
            total_commitment_size_bytes: total_commitment_size,
            total_verified_size_bytes: total_verified_size,
            total_verified_size_gb: total_verified_size as f64 / (1024.0 * 1024.0 * 1024.0),
        },
        completeness_estimates: CompletenessSummary {
            block_completeness_percent: blocks_found as f64 / 1_051_200.0 * 100.0,
            da_completeness_percent: shares_found as f64 / (256_650.0 / 50.0) * 100.0,
        },
        performance: VerificationPerformance {
            verification_time_seconds: verification_time.as_secs_f64(),
            objects_verified_per_sec: (blocks_found + shares_found + commitments_found) as f64
                / verification_time.as_secs_f64(),
        },
        warnings: WarningsSummary {
            block_generation_incomplete: blocks_found < 1_000_000,
            data_size_smaller_than_expected: total_verified_size < 50 * 1024 * 1024 * 1024,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    match std::fs::write(&filepath, serde_json::to_string_pretty(&report).unwrap()) {
        Ok(_) => info!("Verification report saved to: {}", filepath.display()),
        Err(e) => log::warn!(
            "Failed to save verification report to {}: {}",
            filepath.display(),
            e
        ),
    }
}
