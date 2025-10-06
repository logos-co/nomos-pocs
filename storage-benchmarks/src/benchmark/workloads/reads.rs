use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use nomos_storage::{
    api::{chain::StorageChainApi as _, da::StorageDaApi as _},
    backends::rocksdb::RocksBackend,
};
use tokio::sync::Mutex;

use super::super::{create_blob_id, create_header_id, safe_interval_from_hz, WorkloadStreamResult};
use crate::{
    config::{types::WorkloadType, ValidatorProfile},
    data::{select_block_spec_accurate, select_da_spec_accurate},
    metrics::LatencyTracker,
};

pub async fn run_block_validation_workload(
    storage: Arc<Mutex<RocksBackend>>,
    duration: Duration,
    frequency_hz: f64,
    max_blocks: usize,
    profile: &ValidatorProfile,
) -> WorkloadStreamResult {
    let mut result = WorkloadStreamResult {
        workload_type: WorkloadType::BlockValidation,
        executed: true,
        operations_total: 0,
        operations_success: 0,
        bytes_read: 0,
        bytes_written: 0,
        duration,
        errors: 0,
        cache_misses: 0,
        latency_percentiles: None,
    };

    let mut latency_tracker = LatencyTracker::new();

    let interval = match safe_interval_from_hz(frequency_hz, &result.workload_type.to_string()) {
        Ok(interval) => interval,
        Err(e) => {
            log::warn!("{e}");
            result.duration = duration;
            result.latency_percentiles = Some(latency_tracker.get_percentiles());
            return result;
        }
    };

    let mut ticker = tokio::time::interval(interval);
    let end_time = Instant::now() + duration;

    while Instant::now() < end_time {
        ticker.tick().await;

        let block_index = select_block_spec_accurate(result.operations_total, max_blocks, profile);
        let header_id = create_header_id(block_index);

        let operation_result = latency_tracker
            .record_async_operation(|| async {
                let mut storage_guard = storage.lock().await;
                let get_result = storage_guard.get_block(header_id).await;
                drop(storage_guard);
                get_result
            })
            .await;

        match operation_result {
            Ok(Some(data)) => {
                result.operations_success += 1;
                result.bytes_read += data.len() as u64;
            }
            Ok(None) => {}
            Err(_) => result.errors += 1,
        }

        result.operations_total += 1;
    }

    result.duration = duration;
    result.latency_percentiles = Some(latency_tracker.get_percentiles());
    result
}

pub async fn run_da_sampling_workload(
    storage: Arc<Mutex<RocksBackend>>,
    duration: Duration,
    frequency_hz: f64,
    max_blocks: usize,
    profile: &ValidatorProfile,
) -> WorkloadStreamResult {
    let mut result = WorkloadStreamResult {
        workload_type: WorkloadType::DaSampling,
        executed: true,
        operations_total: 0,
        operations_success: 0,
        bytes_read: 0,
        bytes_written: 0,
        duration,
        errors: 0,
        cache_misses: 0,
        latency_percentiles: None,
    };

    let mut latency_tracker = LatencyTracker::new();

    let interval = match safe_interval_from_hz(frequency_hz, &result.workload_type.to_string()) {
        Ok(interval) => interval,
        Err(e) => {
            log::warn!("{e}");
            result.duration = duration;
            result.latency_percentiles = Some(latency_tracker.get_percentiles());
            return result;
        }
    };

    let mut ticker = tokio::time::interval(interval);
    let end_time = Instant::now() + duration;

    while Instant::now() < end_time {
        ticker.tick().await;

        let blob_index = select_da_spec_accurate(result.operations_total, max_blocks, profile);
        let blob_id = create_blob_id(blob_index, 0);
        let share_idx = [(result.operations_total % 20) as u8, 0u8];

        let operation_result = latency_tracker
            .record_async_operation(|| async {
                let mut storage_guard = storage.lock().await;
                let get_result = storage_guard.get_light_share(blob_id, share_idx).await;
                drop(storage_guard);
                get_result
            })
            .await;

        match operation_result {
            Ok(Some(data)) => {
                result.operations_success += 1;
                result.bytes_read += data.len() as u64;
            }
            Ok(None) => {}
            Err(_) => result.errors += 1,
        }

        result.operations_total += 1;
    }

    result.duration = duration;
    result.latency_percentiles = Some(latency_tracker.get_percentiles());
    result
}

pub async fn run_ibd_serving_workload(
    storage: Arc<Mutex<RocksBackend>>,
    duration: Duration,
    frequency_hz: f64,
    max_blocks: usize,
) -> WorkloadStreamResult {
    const IBD_CHUNK_SIZE: usize = 1000;

    let mut result = WorkloadStreamResult {
        workload_type: WorkloadType::IbdServing,
        executed: true,
        operations_total: 0,
        operations_success: 0,
        bytes_read: 0,
        bytes_written: 0,
        duration,
        errors: 0,
        cache_misses: 0,
        latency_percentiles: None,
    };

    let mut latency_tracker = LatencyTracker::new();

    let interval = match safe_interval_from_hz(frequency_hz, &result.workload_type.to_string()) {
        Ok(interval) => interval,
        Err(e) => {
            log::warn!("{e}");
            result.duration = duration;
            result.latency_percentiles = Some(latency_tracker.get_percentiles());
            return result;
        }
    };

    let mut ticker = tokio::time::interval(interval);
    let end_time = Instant::now() + duration;

    while Instant::now() < end_time {
        ticker.tick().await;

        let max_safe_blocks = max_blocks.saturating_sub(IBD_CHUNK_SIZE).max(1);
        let start_block = (result.operations_total as usize * IBD_CHUNK_SIZE) % max_safe_blocks;
        let start_slot = cryptarchia_engine::Slot::from(start_block as u64);
        let end_slot = cryptarchia_engine::Slot::from((start_block + IBD_CHUNK_SIZE) as u64);
        let Some(limit) = std::num::NonZeroUsize::new(IBD_CHUNK_SIZE) else {
            log::error!("Invalid IBD chunk size: {IBD_CHUNK_SIZE}");
            result.errors += 1;
            continue;
        };

        let operation_result = latency_tracker
            .record_async_operation(|| async {
                let mut storage_guard = storage.lock().await;
                let scan_result = storage_guard
                    .scan_immutable_block_ids(start_slot..=end_slot, limit)
                    .await;

                if let Ok(header_ids) = &scan_result {
                    for header_id in header_ids.iter().take(IBD_CHUNK_SIZE) {
                        let _ = storage_guard.get_block(*header_id).await;
                    }
                }

                drop(storage_guard);
                scan_result
            })
            .await;

        match operation_result {
            Ok(header_ids) => {
                result.operations_success += 1;
                let estimated_bytes = header_ids.len() as u64 * 34371;
                result.bytes_read += estimated_bytes;
            }
            Err(_) => result.errors += 1,
        }
        result.operations_total += 1;
    }

    result.duration = duration;
    result.latency_percentiles = Some(latency_tracker.get_percentiles());
    result
}

pub async fn run_da_commitments_workload(
    storage: Arc<Mutex<RocksBackend>>,
    duration: Duration,
    frequency_hz: f64,
    max_blocks: usize,
    profile: &ValidatorProfile,
) -> WorkloadStreamResult {
    let mut result = WorkloadStreamResult {
        workload_type: WorkloadType::DaCommitments,
        executed: true,
        operations_total: 0,
        operations_success: 0,
        bytes_read: 0,
        bytes_written: 0,
        duration,
        errors: 0,
        cache_misses: 0,
        latency_percentiles: None,
    };

    let mut latency_tracker = LatencyTracker::new();

    let interval = match safe_interval_from_hz(frequency_hz, &result.workload_type.to_string()) {
        Ok(interval) => interval,
        Err(e) => {
            log::warn!("{e}");
            result.duration = duration;
            result.latency_percentiles = Some(latency_tracker.get_percentiles());
            return result;
        }
    };

    let mut ticker = tokio::time::interval(interval);
    let end_time = Instant::now() + duration;

    while Instant::now() < end_time {
        ticker.tick().await;

        let blob_index = select_da_spec_accurate(result.operations_total, max_blocks, profile);
        let blob_id = create_blob_id(blob_index, 0);

        let operation_result = latency_tracker
            .record_async_operation(|| async {
                let mut storage_guard = storage.lock().await;
                let get_result = storage_guard.get_shared_commitments(blob_id).await;
                drop(storage_guard);
                get_result
            })
            .await;

        match operation_result {
            Ok(Some(data)) => {
                result.operations_success += 1;
                result.bytes_read += data.len() as u64;
            }
            Ok(None) => {}
            Err(_) => result.errors += 1,
        }

        result.operations_total += 1;
    }

    result.duration = duration;
    result.latency_percentiles = Some(latency_tracker.get_percentiles());
    result
}
