use std::{sync::Arc, time::Instant};

use log::info;
use nomos_storage::backends::rocksdb::RocksBackend;
use tokio::sync::Mutex;

use super::{
    super::{estimate_sequential_performance, ConcurrentBenchmarkResult},
    reads::{
        run_block_validation_workload, run_da_commitments_workload, run_da_sampling_workload,
        run_ibd_serving_workload,
    },
    writes::{run_conditional_block_storage_workload, run_conditional_da_storage_workload},
};
use crate::{config::ValidatorProfile, metrics::StatsCollector};

pub async fn run_concurrent_validator_benchmark(
    storage: RocksBackend,
    duration: std::time::Duration,
    profile: &ValidatorProfile,
    dataset_size: (usize, usize),
    is_read_only: bool,
) -> Result<ConcurrentBenchmarkResult, Box<dyn std::error::Error>> {
    if is_read_only && (profile.block_write_rate_hz > 0.0 || profile.da_share_write_rate_hz > 0.0) {
        log::warn!("Storage is read-only but profile has write operations. Write workloads will be skipped.");
    }

    let storage = Arc::new(Mutex::new(storage));

    let mut stats_collector = StatsCollector::new();
    stats_collector.collect_before(&*storage.lock().await);

    let start_time = Instant::now();

    info!(
        "Starting concurrent validator simulation for {:.1}s",
        duration.as_secs_f64()
    );
    info!(
        "Network-aware concurrency: {} validators \u{2192} {} IBD streams, {} DA streams",
        profile.total_validators,
        profile.ibd_concurrent_streams(),
        profile.da_concurrent_streams()
    );

    let (
        block_validation_result,
        da_sampling_result,
        da_commitments_result,
        ibd_serving_result,
        block_storage_result,
        da_storage_result,
    ) = tokio::join!(
        run_block_validation_workload(
            Arc::clone(&storage),
            duration,
            profile.block_read_rate_hz,
            dataset_size.0,
            profile
        ),
        run_da_sampling_workload(
            Arc::clone(&storage),
            duration,
            profile.da_share_read_rate_hz,
            dataset_size.0,
            profile
        ),
        run_da_commitments_workload(
            Arc::clone(&storage),
            duration,
            profile.da_share_read_rate_hz * 0.3,
            dataset_size.0,
            profile
        ),
        run_ibd_serving_workload(
            Arc::clone(&storage),
            duration,
            profile.range_scan_rate_hz,
            dataset_size.0
        ),
        run_conditional_block_storage_workload(
            Arc::clone(&storage),
            duration,
            profile.block_write_rate_hz,
            dataset_size.0,
            is_read_only
        ),
        run_conditional_da_storage_workload(
            Arc::clone(&storage),
            duration,
            profile.da_share_write_rate_hz,
            dataset_size.1,
            is_read_only
        )
    );

    let total_duration = start_time.elapsed();

    stats_collector.collect_after(&*storage.lock().await);

    let sequential_estimated_throughput = estimate_sequential_performance(profile);
    let actual_concurrent_throughput = (block_validation_result.operations_success
        + da_sampling_result.operations_success
        + da_commitments_result.operations_success
        + ibd_serving_result.operations_success
        + block_storage_result.operations_success
        + da_storage_result.operations_success) as f64
        / total_duration.as_secs_f64();

    let contention_factor = actual_concurrent_throughput / sequential_estimated_throughput;

    Ok(ConcurrentBenchmarkResult {
        block_validation: block_validation_result,
        da_sampling: da_sampling_result,
        da_commitments: da_commitments_result,
        ibd_serving: ibd_serving_result,
        block_storage: block_storage_result,
        da_storage: da_storage_result,
        total_duration,
        peak_memory_mb: 0.0,
        resource_contention_factor: contention_factor,
        concurrent_operations_peak: 6,
        rocksdb_stats_before: stats_collector.before.clone(),
        rocksdb_stats_after: stats_collector.after.clone(),
    })
}
