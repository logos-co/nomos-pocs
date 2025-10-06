use std::{env, time::Instant};

use log::info;
use nomos_storage::{
    api::chain::StorageChainApi as _,
    backends::{rocksdb::RocksBackend, StorageBackend as _},
};
use serde::{Deserialize, Serialize};
use storage_benchmarks::{
    benchmark::{analyze_dataset, utilities::create_header_id},
    data::create_block_data,
    BenchConfig, DatasetGenConfig,
};

pub struct DatasetGenerator {
    config: DatasetGenConfig,
    storage: RocksBackend,
    progress: GenerationProgress,
    stats: GenerationStats,
}

#[derive(Debug, Clone, Default)]
pub struct GenerationProgress {
    pub blocks_completed: usize,
    pub da_objects_completed: usize,
    pub current_batch_start: usize,
    pub total_target_blocks: usize,
    pub generation_start_time: Option<Instant>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationStats {
    pub blocks_generated_this_run: usize,
    pub da_objects_generated_this_run: usize,
    pub total_generation_time: std::time::Duration,
    pub block_generation_rate: f64,
    pub da_generation_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetGenerationReport {
    pub generation_summary: GenerationSummary,
    pub performance: PerformanceMetrics,
    pub config: DatasetGenConfig,
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
pub struct PerformanceMetrics {
    pub block_rate_per_sec: f64,
    pub da_rate_per_sec: f64,
    pub total_rate_objects_per_sec: f64,
    pub cpu_cores_used: usize,
}

impl DatasetGenerator {
    pub async fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = DatasetGenConfig::from_file(config_path)?;
        let benchmark_config = BenchConfig::production();
        let storage = RocksBackend::new(benchmark_config.settings)?;

        let mut generator = Self {
            config,
            storage,
            progress: GenerationProgress::default(),
            stats: GenerationStats::default(),
        };

        generator.analyze_existing_data().await?;

        Ok(generator)
    }

    async fn analyze_existing_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (existing_blocks, existing_da) = analyze_dataset(&mut self.storage).await?;

        self.progress.blocks_completed = existing_blocks;
        self.progress.da_objects_completed = existing_da;
        self.progress.total_target_blocks = self.config.total_blocks();

        info!(
            "Found existing data: {} blocks, {} DA objects",
            existing_blocks, existing_da
        );
        info!("Target: {} total blocks", self.progress.total_target_blocks);

        Ok(())
    }

    pub async fn generate_dataset(
        &mut self,
    ) -> Result<GenerationStats, Box<dyn std::error::Error>> {
        info!(
            "Multi-core generation: {} ({} cores available)",
            self.config.dataset.name,
            num_cpus::get()
        );

        self.progress.generation_start_time = Some(Instant::now());

        if self.progress.blocks_completed < self.progress.total_target_blocks {
            self.generate_remaining_blocks().await?;
        } else {
            info!("All blocks already generated!");
        }

        self.generate_da_objects()?;

        self.finalize_generation();
        self.save_generation_report();

        Ok(self.stats.clone())
    }

    async fn generate_remaining_blocks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let remaining_blocks = self.progress.total_target_blocks - self.progress.blocks_completed;

        info!(
            "Resuming block generation from block {}, generating {} more blocks",
            self.progress.blocks_completed, remaining_blocks
        );

        let blocks_generated = self.generate_blocks_in_batches(remaining_blocks).await?;
        self.stats.blocks_generated_this_run = blocks_generated;

        Ok(())
    }

    async fn generate_blocks_in_batches(
        &mut self,
        blocks_to_generate: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        const PARALLEL_BATCH_SIZE: usize = 1000;
        let mut blocks_generated = 0;

        for batch_start in (0..blocks_to_generate).step_by(PARALLEL_BATCH_SIZE) {
            let batch_end = std::cmp::min(batch_start + PARALLEL_BATCH_SIZE, blocks_to_generate);
            let actual_batch_start = self.progress.blocks_completed + batch_start;

            let batch_data =
                self.generate_block_batch_parallel(actual_batch_start, batch_end - batch_start)?;
            self.store_block_batch(&batch_data).await?;

            blocks_generated += batch_end - batch_start;
            self.log_block_progress(actual_batch_start, blocks_generated);
        }

        Ok(blocks_generated)
    }

    fn generate_block_batch_parallel(
        &self,
        start_index: usize,
        count: usize,
    ) -> Result<Vec<(usize, bytes::Bytes)>, Box<dyn std::error::Error>> {
        use rayon::prelude::*;

        let generation_start = Instant::now();
        let batch_data: Vec<_> = (0..count)
            .into_par_iter()
            .map(|i| {
                let block_index = start_index + i;
                let block_data = create_block_data(block_index, self.config.blocks.size_bytes);
                (block_index, block_data)
            })
            .collect();

        let generation_time = generation_start.elapsed();
        info!(
            "Generated {} blocks in {:.2}s ({:.0} blocks/s)",
            count,
            generation_time.as_secs_f64(),
            count as f64 / generation_time.as_secs_f64()
        );

        Ok(batch_data)
    }

    async fn store_block_batch(
        &mut self,
        batch: &[(usize, bytes::Bytes)],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let storage_start = Instant::now();

        for (block_index, block_data) in batch {
            let header_id = create_header_id(*block_index);
            self.storage
                .store_block(header_id, block_data.clone())
                .await?;

            let slot = cryptarchia_engine::Slot::from(*block_index as u64);
            let ids = std::collections::BTreeMap::from([(slot, header_id)]);
            self.storage.store_immutable_block_ids(ids).await?;
        }

        let storage_time = storage_start.elapsed();
        info!(
            "Stored {} blocks in {:.2}s ({:.0} blocks/s)",
            batch.len(),
            storage_time.as_secs_f64(),
            batch.len() as f64 / storage_time.as_secs_f64()
        );

        Ok(())
    }

    fn generate_da_objects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.stats.da_objects_generated_this_run = 0;
        Ok(())
    }

    fn log_block_progress(&self, current_block: usize, blocks_generated: usize) {
        if self.progress.total_target_blocks > 1000 {
            let completion_percent =
                (blocks_generated * 100) as f64 / self.progress.total_target_blocks as f64;
            info!(
                "Block progress: {} completed - {:.1}% total",
                current_block, completion_percent
            );
        }
    }

    fn finalize_generation(&mut self) {
        if let Some(start_time) = self.progress.generation_start_time {
            self.stats.total_generation_time = start_time.elapsed();

            if self.stats.total_generation_time.as_secs() > 0 {
                self.stats.block_generation_rate = self.stats.blocks_generated_this_run as f64
                    / self.stats.total_generation_time.as_secs_f64();
                self.stats.da_generation_rate = self.stats.da_objects_generated_this_run as f64
                    / self.stats.total_generation_time.as_secs_f64();
            }
        }
    }

    fn save_generation_report(&self) {
        let results_dir = BenchConfig::results_path();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("dataset_generation_{}.json", timestamp);
        let filepath = results_dir.join(filename);

        let report = DatasetGenerationReport {
            generation_summary: GenerationSummary {
                blocks_generated: self.stats.blocks_generated_this_run,
                da_objects_generated: self.stats.da_objects_generated_this_run,
                total_objects: self.stats.blocks_generated_this_run
                    + self.stats.da_objects_generated_this_run,
                duration_seconds: self.stats.total_generation_time.as_secs(),
                duration_minutes: self.stats.total_generation_time.as_secs_f64() / 60.0,
            },
            performance: PerformanceMetrics {
                block_rate_per_sec: self.stats.block_generation_rate,
                da_rate_per_sec: self.stats.da_generation_rate,
                total_rate_objects_per_sec: self.stats.block_generation_rate
                    + self.stats.da_generation_rate,
                cpu_cores_used: num_cpus::get(),
            },
            config: self.config.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        match std::fs::write(&filepath, serde_json::to_string_pretty(&report).unwrap()) {
            Ok(_) => info!("Generation report saved to: {}", filepath.display()),
            Err(e) => log::warn!("Failed to save report to {}: {}", filepath.display(), e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args[1] != "--config" {
        print_usage();
        return Err("Configuration file required".into());
    }

    let mut generator = DatasetGenerator::new(&args[2]).await?;
    let final_stats = generator.generate_dataset().await?;

    info!("Generation completed successfully!");
    info!(
        "Final stats: {} blocks, {} DA objects in {:.1}min",
        final_stats.blocks_generated_this_run,
        final_stats.da_objects_generated_this_run,
        final_stats.total_generation_time.as_secs_f64() / 60.0
    );

    Ok(())
}

fn print_usage() {
    eprintln!("Multi-core Dataset Builder");
    eprintln!("Generates blocks and DA data in parallel");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  cargo run --bin dataset_builder -- --config <file>");
}
