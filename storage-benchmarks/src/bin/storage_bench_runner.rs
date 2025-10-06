use clap::Parser as _;
use log::info;
use nomos_storage::backends::{rocksdb::RocksBackend, StorageBackend as _};
use storage_benchmarks::{
    benchmark::{
        analyze_dataset, run_concurrent_validator_benchmark, BenchConfigSummary,
        BenchResultsSummary, StatisticsSummary, StorageBenchReport,
    },
    config::{ProductionBenchConfig, ValidatorProfiles},
    data::initialize_benchmark_seed,
    metrics::RuntimeValidatorAllocator,
    BenchConfig,
};

#[global_allocator]
static ALLOCATOR: RuntimeValidatorAllocator = RuntimeValidatorAllocator::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = ProductionBenchConfig::parse();
    config.validate()?;

    let _seed_config = initialize_benchmark_seed(&[]);

    run_benchmark(config).await
}

async fn run_benchmark(config: ProductionBenchConfig) -> Result<(), Box<dyn std::error::Error>> {
    ALLOCATOR.set_limit_gb(config.memory as usize);

    let profiles = ValidatorProfiles::from_file("dataset_configs/validator_profiles.toml")?;
    let profile = profiles
        .get_profile(&config.profile.to_string())
        .ok_or_else(|| format!("Profile '{}' not found", config.profile))?;

    let bench_config = BenchConfig::production();
    if !bench_config.settings.db_path.exists() {
        return Err("No dataset found".into());
    }

    let mut results = Vec::new();

    for i in 1..=config.warmup_runs {
        info!("Warmup run {}/{}", i, config.warmup_runs);
        let _ = run_iteration(&bench_config, profile, &config).await;
    }

    for i in 1..=config.measurement_runs {
        info!("Measurement run {}/{}", i, config.measurement_runs);
        let result = run_iteration(&bench_config, profile, &config).await;
        info!("Run {} result: {:.1} ops/sec", i, result);
        results.push(result);
    }

    report_results(&results, &config);

    Ok(())
}

async fn run_iteration(
    bench_config: &BenchConfig,
    profile: &storage_benchmarks::config::ValidatorProfile,
    config: &ProductionBenchConfig,
) -> f64 {
    let mut storage_settings = bench_config.settings.clone();
    storage_settings.read_only = config.read_only;

    match RocksBackend::new(storage_settings) {
        Ok(mut storage) => {
            if let Ok((block_count, share_count)) = analyze_dataset(&mut storage).await {
                if let Ok(result) = run_concurrent_validator_benchmark(
                    storage,
                    std::time::Duration::from_secs(config.duration),
                    profile,
                    (block_count, share_count),
                    config.read_only,
                )
                .await
                {
                    return result.combined_throughput();
                }
            }
        }
        Err(e) => log::error!("Storage error: {}", e),
    }

    0.0
}

fn report_results(results: &[f64], config: &ProductionBenchConfig) {
    save_results_to_file(results, config);
    print_results_summary(results, config);
}

fn save_results_to_file(results: &[f64], config: &ProductionBenchConfig) {
    let results_dir = BenchConfig::results_path();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!(
        "storage_bench_{}_{}_{}gb_{}.json",
        config.profile, timestamp, config.memory, config.duration
    );
    let filepath = results_dir.join(filename);

    let mean = if results.is_empty() {
        0.0
    } else {
        results.iter().sum::<f64>() / results.len() as f64
    };
    let min = results.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = results.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let variability = if mean > 0.0 {
        (max - min) / mean * 100.0
    } else {
        0.0
    };

    let detailed_results = StorageBenchReport {
        benchmark_config: BenchConfigSummary {
            profile: format!("{:?}", config.profile),
            memory_gb: config.memory,
            duration_seconds: config.duration,
            warmup_runs: config.warmup_runs,
            measurement_runs: config.measurement_runs,
        },
        results: BenchResultsSummary {
            raw_measurements: results.to_vec(),
            statistics: StatisticsSummary {
                mean_ops_sec: mean,
                min_ops_sec: min,
                max_ops_sec: max,
                variability_percent: variability,
                sample_count: results.len(),
            },
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let json_content = match serde_json::to_string_pretty(&detailed_results) {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to serialize results: {}", e);
            return;
        }
    };

    match std::fs::write(&filepath, json_content) {
        Ok(_) => log::info!("Results saved to: {}", filepath.display()),
        Err(e) => log::warn!("Failed to save results to {}: {}", filepath.display(), e),
    }
}

fn print_results_summary(results: &[f64], config: &ProductionBenchConfig) {
    if results.is_empty() {
        return;
    }

    let mean = results.iter().sum::<f64>() / results.len() as f64;
    let min = results.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = results.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let variability = if mean > 0.0 {
        (max - min) / mean * 100.0
    } else {
        0.0
    };

    info!(
        "Mean: {:.1} ops/sec, Range: {:.1}-{:.1}, Variability: {:.1}%",
        mean, min, max, variability
    );

    let summary = StatisticsSummary {
        mean_ops_sec: mean,
        min_ops_sec: min,
        max_ops_sec: max,
        variability_percent: variability,
        sample_count: results.len(),
    };

    log::info!(
        "MACHINE_READABLE: {}",
        serde_json::to_string(&summary).unwrap_or_default()
    );

    println!("\n| Profile | Memory | Ops/sec | Variability |");
    println!("|---------|--------|---------|-------------|");
    println!(
        "| {} | {}GB | {:.1} | {:.1}% |",
        config.profile, config.memory, mean, variability
    );
}
