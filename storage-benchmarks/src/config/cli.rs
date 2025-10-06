use clap::Parser;

use super::types::{CompressionType, ProfileType};
use crate::RocksDbTuningOptions;

#[derive(Debug, Clone, Parser)]
#[command(name = "optimization_bench")]
#[command(about = "RocksDB optimization benchmarks")]
#[command(long_about = "Systematic RocksDB parameter optimization with statistical rigor")]
#[non_exhaustive]
pub struct ProductionBenchConfig {
    #[arg(long)]
    pub profile: ProfileType,

    #[arg(long, default_value = "8")]
    pub memory: u32,

    #[arg(long, default_value = "120")]
    pub duration: u64,

    #[arg(long)]
    pub cache_size: Option<u32>,

    #[arg(long)]
    pub write_buffer: Option<u32>,

    #[arg(long)]
    pub compaction_jobs: Option<u32>,

    #[arg(long)]
    pub block_size: Option<u32>,

    #[arg(long)]
    pub compression: Option<CompressionType>,

    #[arg(long)]
    pub read_only: bool,

    #[arg(long)]
    pub seed: Option<u64>,

    #[arg(long, default_value = "1")]
    pub warmup_runs: usize,

    #[arg(long, default_value = "3")]
    pub measurement_runs: usize,
}

#[derive(Debug, Clone, Parser)]
#[command(name = "dataset_generator")]
#[command(about = "Multi-core dataset generation")]
pub struct DatasetGeneratorConfig {
    #[arg(long)]
    pub config: std::path::PathBuf,

    #[arg(long)]
    pub seed: Option<u64>,

    #[arg(long)]
    pub size_limit: Option<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Memory limit must be between 1-512GB, got {0}GB")]
    InvalidMemoryLimit(u32),

    #[error("Duration must be between 1-86400 seconds, got {0}s")]
    InvalidDuration(u64),

    #[error("Cache size must be between 1-80% of RAM, got {0}%")]
    InvalidCacheSize(u32),

    #[error("Write buffer must be between 16-2048MB, got {0}MB")]
    InvalidWriteBuffer(u32),

    #[error("Compaction jobs must be between 1-32, got {0}")]
    InvalidCompactionJobs(u32),

    #[error("Block size must be between 1-128KB, got {0}KB")]
    InvalidBlockSize(u32),

    #[error("Warmup runs must be less than measurement runs, got warmup={0}, measurement={1}")]
    InvalidRunCounts(usize, usize),

    #[error("Unknown compression type: {0} (valid: none, lz4, zstd)")]
    InvalidCompression(String),

    #[error("Profile '{0}' not found in validator_profiles.toml")]
    ProfileNotFound(String),
}

impl ProductionBenchConfig {
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if !(1..=512).contains(&self.memory) {
            return Err(ConfigValidationError::InvalidMemoryLimit(self.memory));
        }

        if !(1..=86400).contains(&self.duration) {
            return Err(ConfigValidationError::InvalidDuration(self.duration));
        }

        if let Some(cache) = self.cache_size {
            if !(1..=80).contains(&cache) {
                return Err(ConfigValidationError::InvalidCacheSize(cache));
            }
        }

        if let Some(buffer) = self.write_buffer {
            if !(16..=2048).contains(&buffer) {
                return Err(ConfigValidationError::InvalidWriteBuffer(buffer));
            }
        }

        if let Some(jobs) = self.compaction_jobs {
            if !(1..=32).contains(&jobs) {
                return Err(ConfigValidationError::InvalidCompactionJobs(jobs));
            }
        }

        if let Some(block_size) = self.block_size {
            if !(1..=128).contains(&block_size) {
                return Err(ConfigValidationError::InvalidBlockSize(block_size));
            }
        }

        if self.warmup_runs >= self.measurement_runs {
            return Err(ConfigValidationError::InvalidRunCounts(
                self.warmup_runs,
                self.measurement_runs,
            ));
        }

        if let Some(comp) = self.compression {
            log::debug!("Compression type: {comp}");
        }

        Ok(())
    }

    #[must_use]
    pub const fn to_rocksdb_tuning(&self) -> RocksDbTuningOptions {
        RocksDbTuningOptions {
            cache_size_percent: self.cache_size,
            write_buffer_mb: self.write_buffer,
            compaction_jobs: self.compaction_jobs,
            block_size_kb: self.block_size,
            compression: self.compression,
            bloom_filter_bits: None,
        }
    }
}
