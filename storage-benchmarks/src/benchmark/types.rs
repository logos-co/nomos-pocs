use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::types::WorkloadType,
    metrics::{LatencyPercentiles, RocksDbStats},
};

#[derive(Debug, Clone)]
pub struct WorkloadStreamResult {
    pub workload_type: WorkloadType,

    pub executed: bool,

    pub operations_total: u64,

    pub operations_success: u64,

    pub bytes_read: u64,

    pub bytes_written: u64,

    pub duration: Duration,

    pub errors: u64,

    pub cache_misses: u64,

    pub latency_percentiles: Option<LatencyPercentiles>,
}

#[derive(Debug, Clone)]
pub struct ConcurrentBenchmarkResult {
    pub block_validation: WorkloadStreamResult,
    pub da_sampling: WorkloadStreamResult,

    pub da_commitments: WorkloadStreamResult,
    pub ibd_serving: WorkloadStreamResult,
    pub block_storage: WorkloadStreamResult,
    pub da_storage: WorkloadStreamResult,

    pub total_duration: Duration,

    pub peak_memory_mb: f64,

    pub resource_contention_factor: f64,

    pub concurrent_operations_peak: u64,

    pub rocksdb_stats_before: RocksDbStats,

    pub rocksdb_stats_after: RocksDbStats,
}

impl ConcurrentBenchmarkResult {
    #[must_use]
    pub const fn total_operations(&self) -> u64 {
        let mut total = 0;
        if self.block_validation.executed {
            total += self.block_validation.operations_total;
        }
        if self.da_sampling.executed {
            total += self.da_sampling.operations_total;
        }
        if self.da_commitments.executed {
            total += self.da_commitments.operations_total;
        }
        if self.ibd_serving.executed {
            total += self.ibd_serving.operations_total;
        }
        if self.block_storage.executed {
            total += self.block_storage.operations_total;
        }
        if self.da_storage.executed {
            total += self.da_storage.operations_total;
        }
        total
    }

    #[must_use]
    pub const fn total_success(&self) -> u64 {
        let mut total = 0;
        if self.block_validation.executed {
            total += self.block_validation.operations_success;
        }
        if self.da_sampling.executed {
            total += self.da_sampling.operations_success;
        }
        if self.da_commitments.executed {
            total += self.da_commitments.operations_success;
        }
        if self.ibd_serving.executed {
            total += self.ibd_serving.operations_success;
        }
        if self.block_storage.executed {
            total += self.block_storage.operations_success;
        }
        if self.da_storage.executed {
            total += self.da_storage.operations_success;
        }
        total
    }

    #[must_use]
    pub fn combined_throughput(&self) -> f64 {
        self.total_success() as f64 / self.total_duration.as_secs_f64()
    }

    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_operations() > 0 {
            self.total_success() as f64 / self.total_operations() as f64
        } else {
            0.0
        }
    }

    #[must_use]
    pub fn total_data_throughput_mbps(&self) -> f64 {
        let mut total_bytes = 0;
        if self.block_validation.executed {
            total_bytes += self.block_validation.bytes_read;
        }
        if self.da_sampling.executed {
            total_bytes += self.da_sampling.bytes_read;
        }
        if self.da_commitments.executed {
            total_bytes += self.da_commitments.bytes_read;
        }
        if self.ibd_serving.executed {
            total_bytes += self.ibd_serving.bytes_read;
        }
        if self.block_storage.executed {
            total_bytes += self.block_storage.bytes_written;
        }
        if self.da_storage.executed {
            total_bytes += self.da_storage.bytes_written;
        }
        total_bytes as f64 / 1024.0 / 1024.0 / self.total_duration.as_secs_f64()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageBenchReport {
    pub benchmark_config: BenchConfigSummary,
    pub results: BenchResultsSummary,
    pub timestamp: String,
    pub tool_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchConfigSummary {
    pub profile: String,
    pub memory_gb: u32,
    pub duration_seconds: u64,
    pub warmup_runs: usize,
    pub measurement_runs: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchResultsSummary {
    pub raw_measurements: Vec<f64>,
    pub statistics: StatisticsSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsSummary {
    pub mean_ops_sec: f64,
    pub min_ops_sec: f64,
    pub max_ops_sec: f64,
    pub variability_percent: f64,
    pub sample_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetGenerationReport {
    pub generation_summary: GenerationSummary,
    pub performance: GenerationPerformance,
    pub timestamp: String,
    pub tool_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationSummary {
    pub blocks_generated: usize,
    pub da_objects_generated: usize,
    pub total_objects: usize,
    pub duration_seconds: u64,
    pub duration_minutes: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationPerformance {
    pub total_rate_objects_per_sec: f64,
    pub block_rate_per_sec: f64,
    pub da_rate_per_sec: f64,
    pub cpu_cores_used: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetVerificationReport {
    pub verification_summary: VerificationSummary,
    pub data_sizes: DataSizesSummary,
    pub completeness_estimates: CompletenessSummary,
    pub performance: VerificationPerformance,
    pub warnings: WarningsSummary,
    pub timestamp: String,
    pub tool_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub blocks_found: usize,
    pub da_shares_found: usize,
    pub da_commitments_found: usize,
    pub total_objects_found: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSizesSummary {
    pub total_block_size_bytes: u64,
    pub total_share_size_bytes: u64,
    pub total_commitment_size_bytes: u64,
    pub total_verified_size_bytes: u64,
    pub total_verified_size_gb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletenessSummary {
    pub block_completeness_percent: f64,
    pub da_completeness_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationPerformance {
    pub verification_time_seconds: f64,
    pub objects_verified_per_sec: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WarningsSummary {
    pub block_generation_incomplete: bool,
    pub data_size_smaller_than_expected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub config_summary: BenchConfigSummary,
    pub results: BenchmarkResultsSummary,
    pub metadata: ReportMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResultsSummary {
    pub raw_measurements: Vec<f64>,
    pub warmup_results: Vec<f64>,
    pub statistics: StatisticsSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub timestamp: String,
    pub tool_version: String,
    pub runner_type: String,
}
