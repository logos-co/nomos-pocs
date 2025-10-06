use std::time::Duration;

use log::info;
use nomos_storage::backends::{rocksdb::RocksBackend, StorageBackend as _};
use serde::{Deserialize, Serialize};

use super::{
    analyze_dataset, run_concurrent_validator_benchmark, BenchConfigSummary, BenchmarkReport,
    BenchmarkResultsSummary, ConcurrentBenchmarkResult, ReportMetadata, StatisticsSummary,
};
use crate::{
    config::{ProductionBenchConfig, ValidatorProfile, ValidatorProfiles},
    BenchConfig,
};

pub struct BenchmarkRunner {
    config: ProductionBenchConfig,
    profile: ValidatorProfile,
    storage_config: BenchConfig,
    execution_state: ExecutionState,
    results: BenchmarkResults,
}

#[derive(Debug, Clone, Default)]
struct ExecutionState {
    warmup_completed: usize,
    measurements_completed: usize,
    dataset_size: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Default)]
pub struct BenchmarkResults {
    pub raw_measurements: Vec<f64>,
    pub warmup_results: Vec<f64>,
    pub detailed_results: Vec<ConcurrentBenchmarkResult>,
    pub mean_ops_sec: f64,
    pub variability_percent: f64,
    pub best_result: Option<ConcurrentBenchmarkResult>,
    pub stats_summary: Option<RocksDbStatsSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocksDbStatsSummary {
    pub cache_hit_rate_improvement: f64,
    pub l0_file_growth: i64,
    pub compaction_activity: u64,
    pub memory_usage_change: i64,
}

impl BenchmarkRunner {
    pub fn new(config: ProductionBenchConfig) -> Result<Self, Box<dyn std::error::Error>> {
        config.validate()?;

        let profiles = ValidatorProfiles::from_file("dataset_configs/validator_profiles.toml")?;
        let profile = profiles
            .get_profile(&config.profile.to_string())
            .ok_or_else(|| format!("Profile '{}' not found", config.profile))?
            .clone();

        let storage_config = BenchConfig::production();
        if !storage_config.settings.db_path.exists() {
            return Err("No dataset found - run dataset_generator first".into());
        }

        Ok(Self {
            config,
            profile,
            storage_config,
            execution_state: ExecutionState::default(),
            results: BenchmarkResults::default(),
        })
    }

    pub async fn execute_benchmark(
        &mut self,
    ) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        self.setup_memory_limits();
        self.analyze_dataset().await?;

        info!("Starting warmup phase: {} runs", self.config.warmup_runs);
        for i in 1..=self.config.warmup_runs {
            info!("Warmup run {}/{}", i, self.config.warmup_runs);
            let result = self.run_single_iteration().await?;
            self.results.warmup_results.push(result);
            self.execution_state.warmup_completed = i;
        }

        info!(
            "Starting measurement phase: {} runs",
            self.config.measurement_runs
        );
        for i in 1..=self.config.measurement_runs {
            info!("Measurement run {}/{}", i, self.config.measurement_runs);
            let result = self.run_single_iteration().await?;
            info!("Run {i} result: {result:.1} ops/sec");
            self.results.raw_measurements.push(result);
            self.execution_state.measurements_completed = i;
        }

        self.calculate_final_statistics();
        self.save_results();

        Ok(self.results.clone())
    }

    fn setup_memory_limits(&self) {
        info!("Setting memory limit to {}GB", self.config.memory);
    }

    async fn analyze_dataset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage_settings = self.storage_config.settings.clone();
        storage_settings.read_only = self.config.read_only;

        let mut storage = RocksBackend::new(storage_settings)?;
        let dataset_size = analyze_dataset(&mut storage).await?;

        self.execution_state.dataset_size = Some(dataset_size);
        info!(
            "Dataset analysis: {} blocks, {} shares",
            dataset_size.0, dataset_size.1
        );

        Ok(())
    }

    async fn run_single_iteration(&mut self) -> Result<f64, Box<dyn std::error::Error>> {
        let mut storage_settings = self.storage_config.settings.clone();
        storage_settings.read_only = self.config.read_only;

        let storage = RocksBackend::new(storage_settings)?;
        let dataset_size = self.execution_state.dataset_size.unwrap_or((0, 0));

        match run_concurrent_validator_benchmark(
            storage,
            Duration::from_secs(self.config.duration),
            &self.profile,
            dataset_size,
            self.config.read_only,
        )
        .await
        {
            Ok(detailed_result) => {
                let throughput = detailed_result.combined_throughput();
                self.results.detailed_results.push(detailed_result);
                Ok(throughput)
            }
            Err(e) => {
                log::error!("Benchmark iteration failed: {e}");
                Ok(0.0)
            }
        }
    }

    fn calculate_final_statistics(&mut self) {
        if self.results.raw_measurements.is_empty() {
            return;
        }

        let mean = self.results.raw_measurements.iter().sum::<f64>()
            / self.results.raw_measurements.len() as f64;
        let min = self
            .results
            .raw_measurements
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self
            .results
            .raw_measurements
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let variability = if mean > 0.0 {
            (max - min) / mean * 100.0
        } else {
            0.0
        };

        self.results.mean_ops_sec = mean;
        self.results.variability_percent = variability;

        if let Some(best_idx) = self
            .results
            .raw_measurements
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(idx, _)| idx)
        {
            self.results.best_result = self.results.detailed_results.get(best_idx).cloned();
        }
    }

    fn save_results(&self) {
        let results_dir = BenchConfig::results_path();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "bench_{}_{}_{}gb_{}.json",
            self.config.profile, timestamp, self.config.memory, self.config.duration
        );
        let filepath = results_dir.join(filename);

        let report = BenchmarkReport {
            config_summary: BenchConfigSummary {
                profile: format!("{:?}", self.config.profile),
                memory_gb: self.config.memory,
                duration_seconds: self.config.duration,
                warmup_runs: self.config.warmup_runs,
                measurement_runs: self.config.measurement_runs,
            },
            results: BenchmarkResultsSummary {
                raw_measurements: self.results.raw_measurements.clone(),
                warmup_results: self.results.warmup_results.clone(),
                statistics: StatisticsSummary {
                    mean_ops_sec: self.results.mean_ops_sec,
                    min_ops_sec: 0.0,
                    max_ops_sec: 0.0,
                    variability_percent: self.results.variability_percent,
                    sample_count: self.results.raw_measurements.len(),
                },
            },
            metadata: ReportMetadata {
                timestamp: chrono::Utc::now().to_rfc3339(),
                tool_version: env!("CARGO_PKG_VERSION").to_owned(),
                runner_type: "batch".to_owned(),
            },
        };

        match std::fs::write(&filepath, serde_json::to_string_pretty(&report).unwrap()) {
            Ok(()) => log::info!(
                "Stateful benchmark results saved to: {}",
                filepath.display()
            ),
            Err(e) => log::warn!("Failed to save results to {}: {}", filepath.display(), e),
        }
    }

    #[must_use]
    pub const fn execution_progress(&self) -> (usize, usize, usize, usize) {
        (
            self.execution_state.warmup_completed,
            self.config.warmup_runs,
            self.execution_state.measurements_completed,
            self.config.measurement_runs,
        )
    }

    #[must_use]
    pub const fn current_results(&self) -> &BenchmarkResults {
        &self.results
    }
}
