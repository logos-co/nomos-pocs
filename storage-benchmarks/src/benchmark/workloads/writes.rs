use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use nomos_storage::{
    api::{chain::StorageChainApi as _, da::StorageDaApi as _},
    backends::rocksdb::RocksBackend,
};
use tokio::sync::Mutex;

use super::super::{
    safe_interval_from_hz,
    utilities::{create_blob_id, create_header_id},
    WorkloadStreamResult,
};
use crate::{
    config::types::WorkloadType,
    data::{create_block_data, create_commitment, create_da_share},
    metrics::LatencyTracker,
};

pub async fn run_block_storage_workload(
    storage: Arc<Mutex<RocksBackend>>,
    duration: Duration,
    frequency_hz: f64,
    starting_block_height: usize,
) -> WorkloadStreamResult {
    let mut result = WorkloadStreamResult {
        workload_type: WorkloadType::BlockStorage,
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
    let mut current_height = starting_block_height;

    while Instant::now() < end_time {
        ticker.tick().await;

        let header_id = create_header_id(current_height);
        let block_data = create_block_data(current_height, 34_371);

        let operation_result = latency_tracker
            .record_async_operation(|| async {
                let mut storage_guard = storage.lock().await;
                let store_result = storage_guard
                    .store_block(header_id, block_data.clone())
                    .await;

                if store_result.is_ok() {
                    let slot = cryptarchia_engine::Slot::from(current_height as u64);
                    let ids = std::collections::BTreeMap::from([(slot, header_id)]);
                    let _ = storage_guard.store_immutable_block_ids(ids).await;
                }

                drop(storage_guard);
                store_result
            })
            .await;

        match operation_result {
            Ok(()) => {
                result.operations_success += 1;
                result.bytes_written += block_data.len() as u64;
                current_height += 1;
            }
            Err(_) => result.errors += 1,
        }

        result.operations_total += 1;
    }

    result.duration = duration;
    result.latency_percentiles = Some(latency_tracker.get_percentiles());
    result
}

pub async fn run_da_storage_workload(
    storage: Arc<Mutex<RocksBackend>>,
    duration: Duration,
    frequency_hz: f64,
    starting_share_count: usize,
) -> WorkloadStreamResult {
    let mut result = WorkloadStreamResult {
        workload_type: WorkloadType::DaStorage,
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

        let blob_id = create_blob_id(starting_share_count + result.operations_total as usize, 0);
        let share_idx = [(result.operations_total % 20) as u8, 0u8];
        let share_data = create_da_share(
            starting_share_count + result.operations_total as usize,
            0,
            1024,
        );

        let operation_result = latency_tracker
            .record_async_operation(|| async {
                let mut storage_guard = storage.lock().await;
                let store_result = storage_guard
                    .store_light_share(blob_id, share_idx, share_data.clone())
                    .await;

                if store_result.is_ok() {
                    if let Ok(commitment) = create_commitment(
                        starting_share_count + result.operations_total as usize,
                        0,
                        220_000,
                    )
                    .await
                    {
                        let _ = storage_guard
                            .store_shared_commitments(blob_id, commitment)
                            .await;
                    }
                }

                drop(storage_guard);
                store_result
            })
            .await;

        match operation_result {
            Ok(()) => {
                result.operations_success += 1;
                result.bytes_written += share_data.len() as u64 + 220_000;
            }
            Err(_) => result.errors += 1,
        }

        result.operations_total += 1;
    }

    result.duration = duration;
    result.latency_percentiles = Some(latency_tracker.get_percentiles());
    result
}

pub async fn run_conditional_block_storage_workload(
    storage: Arc<Mutex<RocksBackend>>,
    duration: Duration,
    frequency_hz: f64,
    starting_block_height: usize,
    is_read_only: bool,
) -> WorkloadStreamResult {
    if is_read_only || frequency_hz == 0.0 {
        return create_empty_workload_result(WorkloadType::BlockStorage);
    }

    run_block_storage_workload(storage, duration, frequency_hz, starting_block_height).await
}

pub async fn run_conditional_da_storage_workload(
    storage: Arc<Mutex<RocksBackend>>,
    duration: Duration,
    frequency_hz: f64,
    starting_share_count: usize,
    is_read_only: bool,
) -> WorkloadStreamResult {
    if is_read_only || frequency_hz == 0.0 {
        return create_empty_workload_result(WorkloadType::DaStorage);
    }

    run_da_storage_workload(storage, duration, frequency_hz, starting_share_count).await
}

const fn create_empty_workload_result(workload_type: WorkloadType) -> WorkloadStreamResult {
    WorkloadStreamResult {
        workload_type,
        executed: false,
        operations_total: 0,
        operations_success: 0,
        bytes_read: 0,
        bytes_written: 0,
        duration: Duration::from_secs(0),
        errors: 0,
        cache_misses: 0,
        latency_percentiles: None,
    }
}
