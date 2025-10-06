use std::{env, time::Instant};

use log::info;
use nomos_storage::{
    api::{chain::StorageChainApi as _, da::StorageDaApi as _},
    backends::{rocksdb::RocksBackend, StorageBackend as _},
};
use rand::SeedableRng as _;
use rayon::prelude::*;
use storage_benchmarks::{
    benchmark::{
        analyze_dataset,
        utilities::{create_blob_id, create_header_id},
        DatasetGenerationReport, GenerationPerformance, GenerationSummary,
    },
    data::{create_block_data, create_da_share},
    BenchConfig, DatasetGenConfig,
};

const PARALLEL_BATCH_SIZE: usize = 1000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args[1] != "--config" {
        print_usage();
        return Err("Configuration file required".into());
    }

    run_multicore_generation(&args[2]).await
}

async fn run_multicore_generation(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = DatasetGenConfig::from_file(config_path)?;

    info!(
        "Multi-core generation: {} ({} cores available)",
        config.dataset.name,
        num_cpus::get()
    );

    let generation_start = Instant::now();
    let benchmark_config = BenchConfig::production();
    let mut storage = RocksBackend::new(benchmark_config.settings)?;

    let (existing_blocks, existing_da) = analyze_dataset(&mut storage).await?;
    let total_blocks = config.total_blocks();

    info!(
        "Found existing data: {} blocks, {} DA objects",
        existing_blocks, existing_da
    );
    info!("Target: {} total blocks", total_blocks);

    let blocks_generated = if existing_blocks < total_blocks {
        let remaining_blocks = total_blocks - existing_blocks;
        info!(
            "Resuming block generation from block {}, generating {} more blocks",
            existing_blocks, remaining_blocks
        );
        generate_blocks_multicore(&mut storage, &config, remaining_blocks, existing_blocks).await?
    } else {
        info!("All blocks already generated!");
        0
    };

    let da_generated = generate_da_objects_multicore(&mut storage, &config, total_blocks).await?;

    let total_time = generation_start.elapsed();

    log_generation_completion(blocks_generated, da_generated, total_time);

    Ok(())
}

async fn generate_blocks_multicore(
    storage: &mut RocksBackend,
    config: &DatasetGenConfig,
    blocks_to_generate: usize,
    start_from_block: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut blocks_generated = 0;

    for batch_start in (0..blocks_to_generate).step_by(PARALLEL_BATCH_SIZE) {
        let batch_end = std::cmp::min(batch_start + PARALLEL_BATCH_SIZE, blocks_to_generate);
        let batch_size = batch_end - batch_start;

        let actual_batch_start = start_from_block + batch_start;
        let actual_batch_end = start_from_block + batch_end;

        let block_data_batch =
            generate_block_batch_parallel(actual_batch_start, actual_batch_end, config)?;

        store_block_batch(storage, &block_data_batch).await?;

        blocks_generated += batch_size;

        log_block_progress(
            actual_batch_start,
            actual_batch_end,
            start_from_block + blocks_to_generate,
            blocks_generated,
        );
    }

    Ok(blocks_generated)
}

async fn generate_da_objects_multicore(
    storage: &mut RocksBackend,
    config: &DatasetGenConfig,
    total_blocks: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    info!(
        "Generating DA objects using {} CPU cores...",
        num_cpus::get()
    );
    let mut da_objects_generated = 0;

    for batch_start in (0..total_blocks).step_by(PARALLEL_BATCH_SIZE) {
        let batch_end = std::cmp::min(batch_start + PARALLEL_BATCH_SIZE, total_blocks);

        let da_batch_count =
            generate_da_batch_for_blocks(storage, config, batch_start, batch_end).await?;

        da_objects_generated += da_batch_count;
    }

    Ok(da_objects_generated)
}

fn generate_block_batch_parallel(
    batch_start: usize,
    batch_end: usize,
    config: &DatasetGenConfig,
) -> Result<Vec<(usize, bytes::Bytes)>, Box<dyn std::error::Error>> {
    let batch_indices: Vec<usize> = (batch_start..batch_end).collect();

    let generation_start = Instant::now();
    let block_data_batch: Vec<(usize, bytes::Bytes)> = batch_indices
        .par_iter()
        .map(|&block_index| {
            let block_data = create_block_data(block_index, config.blocks.size_bytes);
            (block_index, block_data)
        })
        .collect();

    let generation_time = generation_start.elapsed();
    info!(
        "Generated {} blocks in {:.2}s ({:.0} blocks/s)",
        batch_end - batch_start,
        generation_time.as_secs_f64(),
        (batch_end - batch_start) as f64 / generation_time.as_secs_f64()
    );

    Ok(block_data_batch)
}

async fn store_block_batch(
    storage: &mut RocksBackend,
    block_batch: &[(usize, bytes::Bytes)],
) -> Result<(), Box<dyn std::error::Error>> {
    let storage_start = Instant::now();

    for (block_index, block_data) in block_batch {
        let header_id = create_header_id(*block_index);

        storage.store_block(header_id, block_data.clone()).await?;

        let slot = cryptarchia_engine::Slot::from(*block_index as u64);
        let ids = std::collections::BTreeMap::from([(slot, header_id)]);
        storage.store_immutable_block_ids(ids).await?;
    }

    let storage_time = storage_start.elapsed();
    info!(
        "Stored {} blocks in {:.2}s ({:.0} blocks/s)",
        block_batch.len(),
        storage_time.as_secs_f64(),
        block_batch.len() as f64 / storage_time.as_secs_f64()
    );

    Ok(())
}

async fn generate_da_batch_for_blocks(
    storage: &mut RocksBackend,
    config: &DatasetGenConfig,
    batch_start: usize,
    batch_end: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let da_specs = collect_da_specs_for_blocks(config, batch_start, batch_end);

    if da_specs.is_empty() {
        return Ok(0);
    }

    let da_data_batch = generate_da_data_parallel(&da_specs, config)?;

    store_da_batch(storage, &da_data_batch).await?;

    Ok(da_data_batch.len())
}

fn collect_da_specs_for_blocks(
    config: &DatasetGenConfig,
    batch_start: usize,
    batch_end: usize,
) -> Vec<(usize, usize, usize)> {
    let mut da_specs = Vec::new();

    for block in batch_start..batch_end {
        for blob in 0..config.network.blobs_per_block {
            let blob_global_index = block * config.network.blobs_per_block + blob;
            let subnet = blob_global_index % config.network.total_subnets;

            if subnet < config.validator.assigned_subnets {
                da_specs.push((block, blob, subnet));
            }
        }
    }

    da_specs
}

fn generate_da_data_parallel(
    da_specs: &[(usize, usize, usize)],
    config: &DatasetGenConfig,
) -> Result<
    Vec<(nomos_core::da::BlobId, [u8; 2], bytes::Bytes, bytes::Bytes)>,
    Box<dyn std::error::Error>,
> {
    let generation_start = Instant::now();

    let da_data_batch: Vec<_> = da_specs
        .par_iter()
        .map(|&(block, blob, subnet)| {
            let blob_id = create_blob_id(block, blob);
            let share_idx = [subnet as u8, 0u8];
            let share_data = create_da_share(block, blob, config.da.share_size_bytes);

            let commitment_data = {
                let mut rng =
                    rand_chacha::ChaCha20Rng::seed_from_u64((block as u64 * 1000) + blob as u64);
                use rand::Rng as _;
                let data: Vec<u8> = (0..config.da.commitment_size_bytes)
                    .map(|_| rng.gen())
                    .collect();
                bytes::Bytes::from(data)
            };

            (blob_id, share_idx, share_data, commitment_data)
        })
        .collect();

    let generation_time = generation_start.elapsed();
    info!(
        "Generated {} DA objects in {:.2}s ({:.0} objects/s)",
        da_data_batch.len(),
        generation_time.as_secs_f64(),
        da_data_batch.len() as f64 / generation_time.as_secs_f64()
    );

    Ok(da_data_batch)
}

async fn store_da_batch(
    storage: &mut RocksBackend,
    da_batch: &[(nomos_core::da::BlobId, [u8; 2], bytes::Bytes, bytes::Bytes)],
) -> Result<(), Box<dyn std::error::Error>> {
    let storage_start = Instant::now();

    for (blob_id, share_idx, share_data, commitment_data) in da_batch {
        storage
            .store_light_share(*blob_id, *share_idx, share_data.clone())
            .await?;
        storage
            .store_shared_commitments(*blob_id, commitment_data.clone())
            .await?;
    }

    let storage_time = storage_start.elapsed();
    info!(
        "Stored {} DA objects in {:.2}s ({:.0} objects/s)",
        da_batch.len(),
        storage_time.as_secs_f64(),
        da_batch.len() as f64 / storage_time.as_secs_f64()
    );

    Ok(())
}

fn log_block_progress(
    batch_start: usize,
    batch_end: usize,
    total_blocks: usize,
    blocks_generated: usize,
) {
    if total_blocks > 1000 {
        info!(
            "Block progress: {}-{} completed - {:.1}% total",
            batch_start,
            batch_end - 1,
            (blocks_generated * 100) as f64 / total_blocks as f64
        );
    }
}

fn log_generation_completion(
    blocks_generated: usize,
    da_generated: usize,
    total_time: std::time::Duration,
) {
    save_generation_report(blocks_generated, da_generated, total_time);

    info!(
        "Multi-core generation completed: {} blocks, {} DA objects in {:.1}min",
        blocks_generated,
        da_generated,
        total_time.as_secs_f64() / 60.0
    );

    let total_rate = (blocks_generated + da_generated) as f64 / total_time.as_secs_f64();
    info!(
        "Total rate: {:.0} objects/sec using {} CPU cores",
        total_rate,
        num_cpus::get()
    );
}

fn save_generation_report(
    blocks_generated: usize,
    da_generated: usize,
    total_time: std::time::Duration,
) {
    let results_dir = BenchConfig::results_path();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("dataset_generation_{}.json", timestamp);
    let filepath = results_dir.join(filename);

    let report = DatasetGenerationReport {
        generation_summary: GenerationSummary {
            blocks_generated,
            da_objects_generated: da_generated,
            total_objects: blocks_generated + da_generated,
            duration_seconds: total_time.as_secs(),
            duration_minutes: total_time.as_secs_f64() / 60.0,
        },
        performance: GenerationPerformance {
            total_rate_objects_per_sec: (blocks_generated + da_generated) as f64
                / total_time.as_secs_f64(),
            block_rate_per_sec: blocks_generated as f64 / total_time.as_secs_f64(),
            da_rate_per_sec: da_generated as f64 / total_time.as_secs_f64(),
            cpu_cores_used: num_cpus::get(),
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    match std::fs::write(&filepath, serde_json::to_string_pretty(&report).unwrap()) {
        Ok(_) => info!("Generation report saved to: {}", filepath.display()),
        Err(e) => log::warn!("Failed to save report to {}: {}", filepath.display(), e),
    }
}

fn print_usage() {
    eprintln!("Multi-core Dataset Generator");
    eprintln!("Uses all CPU cores for parallel data generation");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  POL_PROOF_DEV_MODE=true cargo run --example multicore_dataset_generator -- --config <file>");
}
