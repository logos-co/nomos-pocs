use std::path::PathBuf;

use nomos_storage::backends::rocksdb::RocksBackendSettings;

pub mod benchmark;
pub mod config;
pub mod data;
pub mod metrics;
pub mod storage;

pub use benchmark::*;
pub use config::{
    CompressionType, DatasetGenConfig, NetworkSize, ProductionBenchConfig, ProfileType,
    ValidatorProfile, ValidatorProfiles, WorkloadType,
};
pub use data::*;
pub use metrics::*;
pub use storage::*;

#[derive(Debug, Clone)]
pub struct BenchStorageConfig {
    pub name: String,
    pub settings: RocksBackendSettings,
}

impl BenchStorageConfig {
    #[must_use]
    pub fn production() -> Self {
        Self {
            name: "production".to_string(),
            settings: RocksBackendSettings {
                db_path: Self::data_path(),
                read_only: false,
                column_family: Some("blocks".to_string()),
            },
        }
    }

    #[must_use]
    pub fn data_path() -> PathBuf {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let data_dir = PathBuf::from(home_dir).join(".nomos_storage_benchmarks");
        let _ = std::fs::create_dir_all(&data_dir);
        data_dir.join("rocksdb_data")
    }

    #[must_use]
    pub fn results_path() -> PathBuf {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let results_dir = PathBuf::from(home_dir)
            .join(".nomos_storage_benchmarks")
            .join("results");
        let _ = std::fs::create_dir_all(&results_dir);
        results_dir
    }
}

pub type BenchConfig = BenchStorageConfig;
