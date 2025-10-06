use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use nomos_storage::{
    api::{chain::StorageChainApi as _, da::StorageDaApi as _},
    backends::rocksdb::RocksBackend,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::super::{create_blob_id, create_header_id, safe_interval_from_hz, WorkloadStreamResult};
use crate::{
    config::{types::WorkloadType, ValidatorProfile},
    data::{select_block_spec_accurate, select_da_spec_accurate},
    metrics::LatencyTracker,
};

#[async_trait]
pub trait WorkloadRunner {
    async fn execute(&mut self, duration: Duration) -> WorkloadStreamResult;
    fn workload_type(&self) -> WorkloadType;
    fn is_read_only(&self) -> bool;
}

pub struct BlockValidationRunner {
    storage: Arc<Mutex<RocksBackend>>,
    profile: ValidatorProfile,
    max_blocks: usize,
    frequency_hz: f64,
    latency_tracker: LatencyTracker,
    execution_stats: WorkloadExecutionStats,
}

pub struct DaSamplingRunner {
    storage: Arc<Mutex<RocksBackend>>,
    profile: ValidatorProfile,
    max_blocks: usize,
    frequency_hz: f64,
    latency_tracker: LatencyTracker,
    execution_stats: WorkloadExecutionStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkloadExecutionStats {
    pub operations_attempted: u64,
    pub operations_successful: u64,
    pub bytes_processed: u64,
    pub errors_encountered: u64,
    pub cache_misses_estimated: u64,
    pub execution_start: Option<chrono::DateTime<chrono::Utc>>,
    pub last_operation_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl BlockValidationRunner {
    pub fn new(
        storage: Arc<Mutex<RocksBackend>>,
        profile: ValidatorProfile,
        max_blocks: usize,
        frequency_hz: f64,
    ) -> Self {
        Self {
            storage,
            profile,
            max_blocks,
            frequency_hz,
            latency_tracker: LatencyTracker::new(),
            execution_stats: WorkloadExecutionStats::default(),
        }
    }

    pub async fn execute(&mut self, duration: Duration) -> WorkloadStreamResult {
        self.execution_stats.execution_start = Some(chrono::Utc::now());

        let interval = match safe_interval_from_hz(self.frequency_hz, "block_validation") {
            Ok(interval) => interval,
            Err(e) => {
                log::warn!("{e}");
                return self.create_error_result(duration);
            }
        };

        let mut ticker = tokio::time::interval(interval);
        let end_time = Instant::now() + duration;

        while Instant::now() < end_time {
            ticker.tick().await;
            self.execute_single_block_validation().await;
        }

        self.create_final_result(duration)
    }

    async fn execute_single_block_validation(&mut self) {
        let block_index = select_block_spec_accurate(
            self.execution_stats.operations_attempted,
            self.max_blocks,
            &self.profile,
        );
        let header_id = create_header_id(block_index);

        let operation_result = self
            .latency_tracker
            .record_async_operation(|| async {
                let mut storage_guard = self.storage.lock().await;
                let result = storage_guard.get_block(header_id).await;
                drop(storage_guard);
                result
            })
            .await;

        match operation_result {
            Ok(Some(data)) => {
                self.execution_stats.operations_successful += 1;
                self.execution_stats.bytes_processed += data.len() as u64;
            }
            Ok(None) => {}
            Err(_) => self.execution_stats.errors_encountered += 1,
        }

        self.execution_stats.operations_attempted += 1;
        self.execution_stats.last_operation_time = Some(chrono::Utc::now());
    }

    fn create_final_result(&self, duration: Duration) -> WorkloadStreamResult {
        WorkloadStreamResult {
            workload_type: WorkloadType::BlockValidation,
            executed: true,
            operations_total: self.execution_stats.operations_attempted,
            operations_success: self.execution_stats.operations_successful,
            bytes_read: self.execution_stats.bytes_processed,
            bytes_written: 0,
            duration,
            errors: self.execution_stats.errors_encountered,
            cache_misses: self.execution_stats.cache_misses_estimated,
            latency_percentiles: Some(self.latency_tracker.get_percentiles()),
        }
    }

    fn create_error_result(&self, duration: Duration) -> WorkloadStreamResult {
        WorkloadStreamResult {
            workload_type: WorkloadType::BlockValidation,
            executed: false,
            operations_total: 0,
            operations_success: 0,
            bytes_read: 0,
            bytes_written: 0,
            duration,
            errors: 1,
            cache_misses: 0,
            latency_percentiles: Some(self.latency_tracker.get_percentiles()),
        }
    }

    #[must_use]
    pub const fn execution_state(&self) -> &WorkloadExecutionStats {
        &self.execution_stats
    }
}

#[async_trait]
impl WorkloadRunner for BlockValidationRunner {
    async fn execute(&mut self, duration: Duration) -> WorkloadStreamResult {
        Self::execute(self, duration).await
    }

    fn workload_type(&self) -> WorkloadType {
        WorkloadType::BlockValidation
    }

    fn is_read_only(&self) -> bool {
        true
    }
}

impl DaSamplingRunner {
    pub fn new(
        storage: Arc<Mutex<RocksBackend>>,
        profile: ValidatorProfile,
        max_blocks: usize,
        frequency_hz: f64,
    ) -> Self {
        Self {
            storage,
            profile,
            max_blocks,
            frequency_hz,
            latency_tracker: LatencyTracker::new(),
            execution_stats: WorkloadExecutionStats::default(),
        }
    }

    pub async fn execute(&mut self, duration: Duration) -> WorkloadStreamResult {
        self.execution_stats.execution_start = Some(chrono::Utc::now());

        let interval = match safe_interval_from_hz(self.frequency_hz, "da_sampling") {
            Ok(interval) => interval,
            Err(e) => {
                log::warn!("{e}");
                return self.create_error_result(duration);
            }
        };

        let mut ticker = tokio::time::interval(interval);
        let end_time = Instant::now() + duration;

        while Instant::now() < end_time {
            ticker.tick().await;
            self.execute_single_da_sample().await;
        }

        self.create_final_result(duration)
    }

    async fn execute_single_da_sample(&mut self) {
        let blob_index = select_da_spec_accurate(
            self.execution_stats.operations_attempted,
            self.max_blocks,
            &self.profile,
        );
        let blob_id = create_blob_id(blob_index, 0);
        let share_idx = [(self.execution_stats.operations_attempted % 20) as u8, 0u8];

        let operation_result = self
            .latency_tracker
            .record_async_operation(|| async {
                let mut storage_guard = self.storage.lock().await;
                let result = storage_guard.get_light_share(blob_id, share_idx).await;
                drop(storage_guard);
                result
            })
            .await;

        match operation_result {
            Ok(Some(data)) => {
                self.execution_stats.operations_successful += 1;
                self.execution_stats.bytes_processed += data.len() as u64;
            }
            Ok(None) => {}
            Err(_) => self.execution_stats.errors_encountered += 1,
        }

        self.execution_stats.operations_attempted += 1;
        self.execution_stats.last_operation_time = Some(chrono::Utc::now());
    }

    fn create_final_result(&self, duration: Duration) -> WorkloadStreamResult {
        WorkloadStreamResult {
            workload_type: WorkloadType::DaSampling,
            executed: true,
            operations_total: self.execution_stats.operations_attempted,
            operations_success: self.execution_stats.operations_successful,
            bytes_read: self.execution_stats.bytes_processed,
            bytes_written: 0,
            duration,
            errors: self.execution_stats.errors_encountered,
            cache_misses: self.execution_stats.cache_misses_estimated,
            latency_percentiles: Some(self.latency_tracker.get_percentiles()),
        }
    }

    fn create_error_result(&self, duration: Duration) -> WorkloadStreamResult {
        WorkloadStreamResult {
            workload_type: WorkloadType::DaSampling,
            executed: false,
            operations_total: 0,
            operations_success: 0,
            bytes_read: 0,
            bytes_written: 0,
            duration,
            errors: 1,
            cache_misses: 0,
            latency_percentiles: Some(self.latency_tracker.get_percentiles()),
        }
    }
}

#[async_trait]
impl WorkloadRunner for DaSamplingRunner {
    async fn execute(&mut self, duration: Duration) -> WorkloadStreamResult {
        Self::execute(self, duration).await
    }

    fn workload_type(&self) -> WorkloadType {
        WorkloadType::DaSampling
    }

    fn is_read_only(&self) -> bool {
        true
    }
}
