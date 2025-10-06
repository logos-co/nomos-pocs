use serde::{Deserialize, Serialize};

use crate::CompressionType;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RocksDbTuningOptions {
    pub cache_size_percent: Option<u32>,

    pub write_buffer_mb: Option<u32>,

    pub compaction_jobs: Option<u32>,

    pub block_size_kb: Option<u32>,

    pub compression: Option<CompressionType>,

    pub bloom_filter_bits: Option<u32>,
}

impl RocksDbTuningOptions {
    pub fn apply_to_options(
        &self,
        opts: &mut rocksdb::Options,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(cache_percent) = self.cache_size_percent {
            let system_memory_gb = get_system_memory_gb();
            let cache_size_bytes = ((system_memory_gb as f64 * (f64::from(cache_percent) / 100.0))
                * 1024.0
                * 1024.0
                * 1024.0) as usize;

            let cache = rocksdb::Cache::new_lru_cache(cache_size_bytes);
            let mut block_opts = rocksdb::BlockBasedOptions::default();
            block_opts.set_block_cache(&cache);
            opts.set_block_based_table_factory(&block_opts);

            log::info!(
                "Applied block cache: {}% of RAM = {}MB",
                cache_percent,
                cache_size_bytes / 1024 / 1024
            );
        }

        if let Some(buffer_mb) = self.write_buffer_mb {
            let buffer_bytes = (buffer_mb as usize) * 1024 * 1024;
            opts.set_write_buffer_size(buffer_bytes);
            log::info!("Applied write buffer: {}MB", buffer_mb);
        }

        if let Some(jobs) = self.compaction_jobs {
            opts.set_max_background_jobs(jobs as i32);
            log::info!("Applied compaction jobs: {}", jobs);
        }

        if let Some(block_size_kb) = self.block_size_kb {
            let block_size_bytes = (block_size_kb as usize) * 1024;
            let mut block_opts = rocksdb::BlockBasedOptions::default();
            block_opts.set_block_size(block_size_bytes);
            opts.set_block_based_table_factory(&block_opts);
            log::info!("Applied block size: {}KB", block_size_kb);
        }

        if let Some(compression) = self.compression {
            match compression {
                CompressionType::None => {
                    opts.set_compression_type(rocksdb::DBCompressionType::None);
                    log::info!("Applied compression: None");
                }
                CompressionType::Lz4 => {
                    opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
                    log::info!("Applied compression: LZ4");
                }
                CompressionType::Snappy => {
                    opts.set_compression_type(rocksdb::DBCompressionType::Snappy);
                    log::info!("Applied compression: Snappy");
                }
                CompressionType::Zstd => {
                    opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
                    log::info!("Applied compression: Zstd");
                }
            }
        }

        Ok(())
    }

    pub fn from_args(args: &[String]) -> (Self, bool) {
        let mut config = Self::default();
        let mut read_only = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--cache-size" if i + 1 < args.len() => {
                    config.cache_size_percent = args[i + 1].parse().ok();
                    i += 2;
                }
                "--write-buffer" if i + 1 < args.len() => {
                    config.write_buffer_mb = args[i + 1].parse().ok();
                    i += 2;
                }
                "--compaction-jobs" if i + 1 < args.len() => {
                    config.compaction_jobs = args[i + 1].parse().ok();
                    i += 2;
                }
                "--block-size" if i + 1 < args.len() => {
                    config.block_size_kb = args[i + 1].parse().ok();
                    i += 2;
                }
                "--read-only" => {
                    read_only = true;
                    i += 1;
                }
                "--compression" if i + 1 < args.len() => {
                    match args[i + 1].parse::<CompressionType>() {
                        Ok(compression_type) => config.compression = Some(compression_type),
                        Err(e) => log::warn!("Invalid compression type: {}", e),
                    }
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        (config, read_only)
    }

    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(cache) = self.cache_size_percent {
            parts.push(format!("cache:{}%", cache));
        }
        if let Some(buffer) = self.write_buffer_mb {
            parts.push(format!("buffer:{}MB", buffer));
        }
        if let Some(jobs) = self.compaction_jobs {
            parts.push(format!("jobs:{}", jobs));
        }
        if let Some(block_size) = self.block_size_kb {
            parts.push(format!("block:{}KB", block_size));
        }

        if parts.is_empty() {
            "defaults".to_string()
        } else {
            parts.join(",")
        }
    }
}

fn get_system_memory_gb() -> usize {
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<usize>() {
                        return kb / 1024 / 1024;
                    }
                }
            }
        }
    }

    16
}

pub fn create_tuned_rocksdb_options(tuning_config: &RocksDbTuningOptions) -> rocksdb::Options {
    let mut opts = rocksdb::Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);

    if let Err(e) = tuning_config.apply_to_options(&mut opts) {
        log::error!("Failed to apply RocksDB tuning: {}", e);
    }

    opts
}
