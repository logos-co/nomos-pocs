use std::collections::HashMap;

use nomos_storage::backends::rocksdb::RocksBackend;
use serde::{Deserialize, Serialize};

pub struct RocksDbStatsCollector {
    storage_ref: Option<*const RocksBackend>,
    property_cache: HashMap<String, Option<u64>>,
    stats_cache: Option<String>,
    cache_valid: bool,
    collection_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocksDbStatsSnapshot {
    pub stats: super::RocksDbStats,
    pub collection_timestamp: chrono::DateTime<chrono::Utc>,
    pub collection_id: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl RocksDbStatsCollector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage_ref: None,
            property_cache: HashMap::new(),
            stats_cache: None,
            cache_valid: false,
            collection_count: 0,
        }
    }

    pub fn attach(&mut self, storage: &RocksBackend) {
        self.storage_ref = Some(std::ptr::from_ref::<RocksBackend>(storage));
        self.invalidate_cache();
    }

    pub fn collect_stats(&mut self) -> Result<RocksDbStatsSnapshot, Box<dyn std::error::Error>> {
        // SAFETY: storage_ref is set in attach() and guaranteed to be valid for the
        // lifetime of this collector
        let storage = unsafe {
            self.storage_ref
                .ok_or("No storage attached")?
                .as_ref()
                .ok_or("Invalid storage ref")?
        };

        self.collection_count += 1;

        let stats = self.collect_with_caching(storage)?;

        Ok(RocksDbStatsSnapshot {
            stats,
            collection_timestamp: chrono::Utc::now(),
            collection_id: self.collection_count,
            cache_hits: self.count_cache_hits(),
            cache_misses: self.count_cache_misses(),
        })
    }

    pub fn collect_before_after<F>(
        &mut self,
        operation: F,
    ) -> Result<(RocksDbStatsSnapshot, RocksDbStatsSnapshot), Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
    {
        let before = self.collect_stats()?;
        self.invalidate_cache();

        operation()?;

        let after = self.collect_stats()?;

        Ok((before, after))
    }

    pub fn invalidate_cache(&mut self) {
        self.property_cache.clear();
        self.stats_cache = None;
        self.cache_valid = false;
    }

    #[must_use]
    pub fn collection_stats(&self) -> (u64, u64, u64) {
        (
            self.collection_count,
            self.count_cache_hits(),
            self.count_cache_misses(),
        )
    }

    fn collect_with_caching(
        &mut self,
        storage: &RocksBackend,
    ) -> Result<super::RocksDbStats, Box<dyn std::error::Error>> {
        let stats_string = if let Some(ref cached) = self.stats_cache {
            cached.clone()
        } else {
            let stats = self.get_stats_string(storage)?;
            self.stats_cache = Some(stats.clone());
            stats
        };

        let (cache_hit_count, cache_miss_count) = self.parse_cache_hit_miss(&stats_string);
        let cache_hit_rate = if cache_hit_count + cache_miss_count > 0 {
            cache_hit_count as f64 / (cache_hit_count + cache_miss_count) as f64
        } else {
            0.0
        };

        let level_files: Vec<u64> = (0..7)
            .map(|level| {
                self.get_cached_property_u64(
                    storage,
                    rocksdb::properties::num_files_at_level(level).as_ref(),
                )
            })
            .collect();

        Ok(super::RocksDbStats {
            cache_hit_rate,
            cache_hit_count,
            cache_miss_count,
            block_cache_usage_bytes: self
                .get_cached_property_u64(storage, rocksdb::properties::BLOCK_CACHE_USAGE.as_ref()),
            block_cache_capacity_bytes: self.get_cached_property_u64(
                storage,
                rocksdb::properties::BLOCK_CACHE_CAPACITY.as_ref(),
            ),
            index_cache_usage_bytes: self.get_cached_property_u64(
                storage,
                rocksdb::properties::ESTIMATE_TABLE_READERS_MEM.as_ref(),
            ),

            compaction_pending_bytes: self.get_cached_property_u64(
                storage,
                rocksdb::properties::ESTIMATE_PENDING_COMPACTION_BYTES.as_ref(),
            ),
            compaction_running_count: self.get_cached_property_u64(
                storage,
                rocksdb::properties::NUM_RUNNING_COMPACTIONS.as_ref(),
            ),

            l0_file_count: level_files[0],
            l1_file_count: level_files[1],
            l2_file_count: level_files[2],
            l3_file_count: level_files[3],
            l4_file_count: level_files[4],
            l5_file_count: level_files[5],
            l6_file_count: level_files[6],
            total_sst_files: level_files.iter().sum(),
            total_sst_size_bytes: self.get_cached_property_u64(
                storage,
                rocksdb::properties::TOTAL_SST_FILES_SIZE.as_ref(),
            ),

            memtable_count: self.parse_memtable_count(&stats_string),
            num_immutable_memtables: self.parse_immutable_memtables(&stats_string),
            memtable_flush_pending: self.get_cached_property_u64(
                storage,
                rocksdb::properties::NUM_RUNNING_FLUSHES.as_ref(),
            ),
            approximate_memory_usage_bytes: self.get_cached_property_u64(
                storage,
                rocksdb::properties::CUR_SIZE_ALL_MEM_TABLES.as_ref(),
            ),

            read_amplification: self.parse_read_amplification(&stats_string),
            write_amplification: self.parse_write_amplification(&stats_string),
            total_read_bytes: self.parse_total_read_bytes(&stats_string),
            total_write_bytes: self.parse_total_write_bytes(&stats_string),
            write_stall_time_ms: self.parse_write_stall_time(&stats_string),

            live_sst_files_size_bytes: self.get_cached_property_u64(
                storage,
                rocksdb::properties::LIVE_SST_FILES_SIZE.as_ref(),
            ),
            num_entries: self
                .get_cached_property_u64(storage, rocksdb::properties::ESTIMATE_NUM_KEYS.as_ref()),
        })
    }

    fn get_cached_property_u64(&mut self, storage: &RocksBackend, property: &str) -> u64 {
        if let Some(cached_value) = self.property_cache.get(property) {
            return cached_value.unwrap_or(0);
        }

        let value = self.query_property_u64(storage, property);
        self.property_cache.insert(property.to_owned(), value);
        value.unwrap_or(0)
    }

    fn query_property_u64(&self, storage: &RocksBackend, property: &str) -> Option<u64> {
        let property_owned = property.to_owned();
        let transaction = storage.txn(move |db| match db.property_value(&property_owned) {
            Ok(Some(value_string)) => Ok(Some(value_string.into_bytes().into())),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        });

        match transaction.execute() {
            Ok(Some(result_bytes)) => {
                let value_str = String::from_utf8_lossy(&result_bytes);
                value_str.trim().parse().ok()
            }
            _ => None,
        }
    }

    fn get_stats_string(
        &self,
        storage: &RocksBackend,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let transaction = storage.txn(|db| match db.property_value(rocksdb::properties::STATS) {
            Ok(Some(stats_string)) => Ok(Some(stats_string.into_bytes().into())),
            _ => Ok(Some(b"".to_vec().into())),
        });

        match transaction.execute() {
            Ok(Some(stats_bytes)) => Ok(String::from_utf8_lossy(&stats_bytes).to_string()),
            _ => Ok(String::new()),
        }
    }

    fn count_cache_hits(&self) -> u64 {
        self.property_cache.values().filter(|v| v.is_some()).count() as u64
    }

    fn count_cache_misses(&self) -> u64 {
        self.property_cache.values().filter(|v| v.is_none()).count() as u64
    }

    fn parse_cache_hit_miss(&self, stats: &str) -> (u64, u64) {
        let mut hits = 0u64;
        let mut misses = 0u64;

        for line in stats.lines() {
            if line.contains("Block cache hit count:") || line.contains("block.cache.hit") {
                if let Some(value) = self.extract_number_from_line(line) {
                    hits = value;
                }
            } else if line.contains("Block cache miss count:") || line.contains("block.cache.miss")
            {
                if let Some(value) = self.extract_number_from_line(line) {
                    misses = value;
                }
            }
        }

        (hits, misses)
    }

    fn parse_memtable_count(&self, stats: &str) -> u64 {
        for line in stats.lines() {
            if line.contains("Number of memtables") || line.contains("num-live-memtables") {
                if let Some(value) = self.extract_number_from_line(line) {
                    return value;
                }
            }
        }
        0
    }

    fn parse_immutable_memtables(&self, stats: &str) -> u64 {
        for line in stats.lines() {
            if line.contains("immutable memtables") || line.contains("num-immutable-mem-table") {
                if let Some(value) = self.extract_number_from_line(line) {
                    return value;
                }
            }
        }
        0
    }

    fn parse_read_amplification(&self, stats: &str) -> f64 {
        for line in stats.lines() {
            if line.contains("read amplification") || line.contains("Read(GB)") {
                if let Some(value) = self.extract_float_from_line(line) {
                    return value;
                }
            }
        }
        0.0
    }

    fn parse_write_amplification(&self, stats: &str) -> f64 {
        for line in stats.lines() {
            if line.contains("write amplification") || line.contains("Write(GB)") {
                if let Some(value) = self.extract_float_from_line(line) {
                    return value;
                }
            }
        }
        0.0
    }

    fn parse_total_read_bytes(&self, stats: &str) -> u64 {
        for line in stats.lines() {
            if line.contains("total bytes read") || line.contains("Read(GB)") {
                if let Some(value) = self.extract_number_from_line(line) {
                    return value;
                }
            }
        }
        0
    }

    fn parse_total_write_bytes(&self, stats: &str) -> u64 {
        for line in stats.lines() {
            if line.contains("total bytes written") || line.contains("Write(GB)") {
                if let Some(value) = self.extract_number_from_line(line) {
                    return value;
                }
            }
        }
        0
    }

    fn parse_write_stall_time(&self, stats: &str) -> u64 {
        for line in stats.lines() {
            if line.contains("Cumulative stall:") && line.contains("H:M:S") {
                if let Some(percent_pos) = line.find("percent") {
                    let before_percent = &line[..percent_pos];
                    if let Some(comma_pos) = before_percent.rfind(',') {
                        let percent_str = before_percent[comma_pos + 1..].trim();
                        if let Ok(percent) = percent_str.parse::<f64>() {
                            return (percent * 10.0) as u64;
                        }
                    }
                }
            }
        }
        0
    }

    fn extract_number_from_line(&self, line: &str) -> Option<u64> {
        if let Some(colon_pos) = line.find(':') {
            let value_part = line[colon_pos + 1..].trim();
            if let Some(number_str) = value_part.split_whitespace().next() {
                let clean_number = number_str.replace(',', "");
                return clean_number.parse().ok();
            }
        }
        None
    }

    fn extract_float_from_line(&self, line: &str) -> Option<f64> {
        if let Some(colon_pos) = line.find(':') {
            let value_part = line[colon_pos + 1..].trim();
            if let Some(number_str) = value_part.split_whitespace().next() {
                return number_str.parse().ok();
            }
        }
        None
    }
}

impl Default for RocksDbStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}
