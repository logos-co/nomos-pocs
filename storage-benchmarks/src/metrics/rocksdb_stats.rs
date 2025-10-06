use nomos_storage::backends::rocksdb::RocksBackend;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocksDbStats {
    pub cache_hit_rate: f64,
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub block_cache_usage_bytes: u64,
    pub block_cache_capacity_bytes: u64,
    pub index_cache_usage_bytes: u64,

    pub compaction_pending_bytes: u64,
    pub compaction_running_count: u64,

    pub l0_file_count: u64,
    pub l1_file_count: u64,
    pub l2_file_count: u64,
    pub l3_file_count: u64,
    pub l4_file_count: u64,
    pub l5_file_count: u64,
    pub l6_file_count: u64,
    pub total_sst_files: u64,
    pub total_sst_size_bytes: u64,

    pub memtable_count: u64,
    pub num_immutable_memtables: u64,
    pub memtable_flush_pending: u64,
    pub approximate_memory_usage_bytes: u64,

    pub read_amplification: f64,
    pub write_amplification: f64,
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
    pub write_stall_time_ms: u64,

    pub live_sst_files_size_bytes: u64,
    pub num_entries: u64,
}

impl Default for RocksDbStats {
    fn default() -> Self {
        Self {
            cache_hit_rate: 0.0,
            cache_hit_count: 0,
            cache_miss_count: 0,
            block_cache_usage_bytes: 0,
            block_cache_capacity_bytes: 0,
            index_cache_usage_bytes: 0,

            compaction_pending_bytes: 0,
            compaction_running_count: 0,

            l0_file_count: 0,
            l1_file_count: 0,
            l2_file_count: 0,
            l3_file_count: 0,
            l4_file_count: 0,
            l5_file_count: 0,
            l6_file_count: 0,
            total_sst_files: 0,
            total_sst_size_bytes: 0,

            memtable_count: 0,
            num_immutable_memtables: 0,
            memtable_flush_pending: 0,
            approximate_memory_usage_bytes: 0,

            read_amplification: 0.0,
            write_amplification: 0.0,
            total_read_bytes: 0,
            total_write_bytes: 0,
            write_stall_time_ms: 0,

            live_sst_files_size_bytes: 0,
            num_entries: 0,
        }
    }
}

#[must_use]
pub fn collect_rocksdb_stats(storage: &RocksBackend) -> RocksDbStats {
    let (cache_hit_count, cache_miss_count) =
        parse_cache_hit_miss_counts(&get_stats_string(storage));
    let cache_hit_rate = if cache_hit_count + cache_miss_count > 0 {
        cache_hit_count as f64 / (cache_hit_count + cache_miss_count) as f64
    } else {
        0.0
    };

    let l0_files = get_level_file_count(storage, 0);
    let l1_files = get_level_file_count(storage, 1);
    let l2_files = get_level_file_count(storage, 2);
    let l3_files = get_level_file_count(storage, 3);
    let l4_files = get_level_file_count(storage, 4);
    let l5_files = get_level_file_count(storage, 5);
    let l6_files = get_level_file_count(storage, 6);

    RocksDbStats {
        cache_hit_rate,
        cache_hit_count,
        cache_miss_count,
        block_cache_usage_bytes: get_property_u64(
            storage,
            &rocksdb::properties::BLOCK_CACHE_USAGE.as_ref(),
        ),
        block_cache_capacity_bytes: get_property_u64(
            storage,
            &rocksdb::properties::BLOCK_CACHE_CAPACITY.as_ref(),
        ),
        index_cache_usage_bytes: get_property_u64(
            storage,
            &rocksdb::properties::ESTIMATE_TABLE_READERS_MEM.as_ref(),
        ),

        compaction_pending_bytes: get_property_u64(
            storage,
            &rocksdb::properties::ESTIMATE_PENDING_COMPACTION_BYTES.as_ref(),
        ),
        compaction_running_count: get_property_u64(
            storage,
            &rocksdb::properties::NUM_RUNNING_COMPACTIONS.as_ref(),
        ),

        l0_file_count: l0_files,
        l1_file_count: l1_files,
        l2_file_count: l2_files,
        l3_file_count: l3_files,
        l4_file_count: l4_files,
        l5_file_count: l5_files,
        l6_file_count: l6_files,
        total_sst_files: l0_files + l1_files + l2_files + l3_files + l4_files + l5_files + l6_files,
        total_sst_size_bytes: get_property_u64(
            storage,
            &rocksdb::properties::TOTAL_SST_FILES_SIZE.as_ref(),
        ),

        memtable_count: parse_memtable_count(&get_stats_string(storage)),
        num_immutable_memtables: parse_immutable_memtables(&get_stats_string(storage)),
        memtable_flush_pending: get_property_u64(
            storage,
            &rocksdb::properties::NUM_RUNNING_FLUSHES.as_ref(),
        ),
        approximate_memory_usage_bytes: get_property_u64(
            storage,
            &rocksdb::properties::CUR_SIZE_ALL_MEM_TABLES.as_ref(),
        ),

        read_amplification: parse_read_amplification(&get_stats_string(storage)),
        write_amplification: parse_write_amplification(&get_stats_string(storage)),
        total_read_bytes: parse_total_read_bytes(&get_stats_string(storage)),
        total_write_bytes: parse_total_write_bytes(&get_stats_string(storage)),
        write_stall_time_ms: parse_write_stall_time(&get_stats_string(storage)),

        live_sst_files_size_bytes: get_property_u64(
            storage,
            &rocksdb::properties::LIVE_SST_FILES_SIZE.as_ref(),
        ),
        num_entries: get_property_u64(storage, &rocksdb::properties::ESTIMATE_NUM_KEYS.as_ref()),
    }
}

fn get_stats_string(storage: &RocksBackend) -> String {
    let transaction = storage.txn(|db| match db.property_value(rocksdb::properties::STATS) {
        Ok(Some(stats_string)) => Ok(Some(stats_string.into_bytes().into())),
        _ => Ok(Some(b"".to_vec().into())),
    });

    match transaction.execute() {
        Ok(Some(stats_bytes)) => String::from_utf8_lossy(&stats_bytes).to_string(),
        _ => String::new(),
    }
}

fn get_level_file_count(storage: &RocksBackend, level: i32) -> u64 {
    get_property_u64(
        storage,
        &rocksdb::properties::num_files_at_level(level as usize).as_ref(),
    )
}

fn get_property_u64(storage: &RocksBackend, property: &str) -> u64 {
    match get_property_value(storage, property) {
        Some(value) => {
            log::debug!("Property '{}': {}", property, value);
            value
        }
        None => {
            log::debug!("Property '{}': unavailable", property);
            0
        }
    }
}

fn get_property_value(storage: &RocksBackend, property: &str) -> Option<u64> {
    let property_owned = property.to_owned();
    let property_for_log = property.to_owned();
    let transaction = storage.txn(move |db| match db.property_value(&property_owned) {
        Ok(Some(value_string)) => Ok(Some(value_string.into_bytes().into())),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    });

    match transaction.execute() {
        Ok(Some(result_bytes)) => {
            let value_str = String::from_utf8_lossy(&result_bytes);
            match value_str.trim().parse::<u64>() {
                Ok(parsed) => {
                    log::trace!("Property '{}' available: {}", property_for_log, parsed);
                    Some(parsed)
                }
                Err(_) => {
                    log::trace!(
                        "Property '{}' parse error from: '{}'",
                        property_for_log,
                        value_str
                    );
                    None
                }
            }
        }
        Ok(None) => {
            log::trace!("Property '{}' unavailable", property_for_log);
            None
        }
        Err(e) => {
            log::trace!("Property '{}' failed: {}", property_for_log, e);
            None
        }
    }
}

fn parse_cache_hit_miss_counts(stats: &str) -> (u64, u64) {
    let mut hits = 0u64;
    let mut misses = 0u64;

    for line in stats.lines() {
        if line.contains("Block cache hit count:") || line.contains("block.cache.hit") {
            if let Some(value) = extract_number_from_line(line) {
                hits = value;
            }
        } else if line.contains("Block cache miss count:") || line.contains("block.cache.miss") {
            if let Some(value) = extract_number_from_line(line) {
                misses = value;
            }
        }
    }

    (hits, misses)
}

fn parse_write_stall_time(stats: &str) -> u64 {
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

fn extract_number_from_line(line: &str) -> Option<u64> {
    if let Some(colon_pos) = line.find(':') {
        let value_part = line[colon_pos + 1..].trim();
        if let Some(number_str) = value_part.split_whitespace().next() {
            let clean_number = number_str.replace(',', "");
            return clean_number.parse().ok();
        }
    }
    None
}

fn parse_read_amplification(stats: &str) -> f64 {
    for line in stats.lines() {
        if line.contains("read amplification") || line.contains("Read(GB)") {
            if let Some(value) = extract_float_from_line(line) {
                return value;
            }
        }
    }
    0.0
}

fn parse_write_amplification(stats: &str) -> f64 {
    for line in stats.lines() {
        if line.contains("write amplification") || line.contains("Write(GB)") {
            if let Some(value) = extract_float_from_line(line) {
                return value;
            }
        }
    }
    0.0
}

fn parse_total_read_bytes(stats: &str) -> u64 {
    for line in stats.lines() {
        if line.contains("total bytes read") || line.contains("Read(GB)") {
            if let Some(value) = extract_number_from_line(line) {
                return value;
            }
        }
    }
    0
}

fn parse_total_write_bytes(stats: &str) -> u64 {
    for line in stats.lines() {
        if line.contains("total bytes written") || line.contains("Write(GB)") {
            if let Some(value) = extract_number_from_line(line) {
                return value;
            }
        }
    }
    0
}

fn parse_memtable_count(stats: &str) -> u64 {
    for line in stats.lines() {
        if line.contains("Number of memtables") || line.contains("num-live-memtables") {
            if let Some(value) = extract_number_from_line(line) {
                return value;
            }
        }
    }
    0
}

fn parse_immutable_memtables(stats: &str) -> u64 {
    for line in stats.lines() {
        if line.contains("immutable memtables") || line.contains("num-immutable-mem-table") {
            if let Some(value) = extract_number_from_line(line) {
                return value;
            }
        }
    }
    0
}

fn extract_float_from_line(line: &str) -> Option<f64> {
    if let Some(colon_pos) = line.find(':') {
        let value_part = line[colon_pos + 1..].trim();
        if let Some(number_str) = value_part.split_whitespace().next() {
            return number_str.parse().ok();
        }
    }
    None
}

pub struct StatsCollector {
    pub before: RocksDbStats,
    pub after: RocksDbStats,
}

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            before: RocksDbStats::default(),
            after: RocksDbStats::default(),
        }
    }

    pub fn collect_before(&mut self, storage: &RocksBackend) {
        self.before = collect_rocksdb_stats(storage);
        log::debug!(
            "Before: cache {:.1}%, L0 files {}",
            self.before.cache_hit_rate * 100.0,
            self.before.l0_file_count
        );
    }

    pub fn collect_after(&mut self, storage: &RocksBackend) {
        self.after = collect_rocksdb_stats(storage);
        log::debug!(
            "After: cache {:.1}%, L0 files {}",
            self.after.cache_hit_rate * 100.0,
            self.after.l0_file_count
        );
    }
}
