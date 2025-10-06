//! Storage capacity estimator
//!
//! Computes block and DA storage requirements for various time periods and
//! network scenarios. Produces summaries, time breakdowns, and simple hardware
//! recommendations.

use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use storage_benchmarks::BenchConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Number of days represented by the period
    pub days: u64,
    /// Human-readable label (e.g., "1 year")
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Block time in seconds
    pub block_time_seconds: u64,
    /// Average block size in bytes
    pub avg_block_size_bytes: u64,
    /// Total DA subnets
    pub total_subnets: u64,
    /// DA share size in bytes
    pub da_share_size_bytes: u64,
    /// DA commitment size in bytes
    pub da_commitment_size_bytes: u64,
    /// Shares per blob
    pub shares_per_blob: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkScenario {
    /// Scenario name
    pub name: String,
    /// Blobs per block
    pub blobs_per_block: u64,
    /// Total validators used to estimate DA responsibility
    pub total_validators: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationConfig {
    /// Time window for the calculation
    pub time_period: TimePeriod,
    /// Network parameters used across scenarios
    pub network: NetworkConfig,
    /// Scenarios to evaluate
    pub scenarios: Vec<NetworkScenario>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockDataResults {
    /// Blocks produced per day
    pub blocks_per_day: u64,
    /// Total blocks in the period
    pub blocks_for_period: u64,
    /// Average block size in KiB
    pub avg_block_size_kb: u64,
    /// Total block data size in GiB for the period
    pub total_block_data_gb: f64,
    /// Period label
    pub time_period_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioResults {
    /// Scenario label
    pub scenario_name: String,
    /// Blobs per block for this scenario
    pub blobs_per_block: u64,
    /// Total validators
    pub total_validators: u64,
    /// Typical subnets assigned per validator
    pub typical_subnets_per_validator: u64,
    /// Percent of subnets likely assigned to a validator
    pub subnet_assignment_percent: f64,
    /// Count of DA shares stored by the validator over the period
    pub shares_stored_count: u64,
    /// Count of blobs assigned over the period
    pub blobs_assigned_count: u64,
    /// DA shares size in GiB
    pub da_shares_gb: f64,
    /// DA commitments size in GiB
    pub da_commitments_gb: f64,
    /// Total DA data size in GiB
    pub total_da_gb: f64,
    /// Total validator storage in GiB (blocks + DA)
    pub total_validator_storage_gb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeBreakdown {
    /// Sequential period index
    pub period_number: u64,
    /// Label (Month/Week/Day N)
    pub period_description: String,
    /// Cumulative storage at this step in GiB
    pub cumulative_gb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareRecommendation {
    /// Scenario label
    pub scenario: String,
    /// Required storage in GiB for the period
    pub storage_gb_for_period: u64,
    /// Recommended device size
    pub recommended_storage: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageCalculationResults {
    /// Input config used to compute results
    pub calculation_config: CalculationConfig,
    /// Aggregate block data for the period
    pub block_data: BlockDataResults,
    /// Per-scenario storage summaries
    pub scenarios: Vec<ScenarioResults>,
    /// Time-based accumulation for visualization
    pub time_breakdown: Vec<TimeBreakdown>,
    /// Simple hardware sizing suggestions
    pub hardware_recommendations: Vec<HardwareRecommendation>,
    /// Notes for stress testing considerations
    pub stress_testing_notes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapacityCalculationReport {
    pub calculation_results: std::collections::HashMap<String, StorageCalculationResults>,
    pub summary: CalculationSummary,
    pub metadata: ReportMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculationSummary {
    pub scenarios_calculated: usize,
    pub total_time_periods: usize,
    pub calculation_timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub tool: String,
    pub version: String,
    pub description: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            block_time_seconds: 30,
            avg_block_size_bytes: 34_371,
            total_subnets: 2048,
            da_share_size_bytes: 1_024,
            da_commitment_size_bytes: 220_000,
            shares_per_blob: 512,
        }
    }
}

impl Default for CalculationConfig {
    fn default() -> Self {
        Self {
            time_period: TimePeriod {
                days: 365,
                description: "1 year".to_string(),
            },
            network: NetworkConfig::default(),
            scenarios: vec![
                NetworkScenario {
                    name: "Conservative".to_string(),
                    blobs_per_block: 50,
                    total_validators: 2000,
                },
                NetworkScenario {
                    name: "Active".to_string(),
                    blobs_per_block: 100,
                    total_validators: 2000,
                },
                NetworkScenario {
                    name: "High Activity".to_string(),
                    blobs_per_block: 200,
                    total_validators: 3000,
                },
                NetworkScenario {
                    name: "Peak".to_string(),
                    blobs_per_block: 500,
                    total_validators: 5000,
                },
            ],
        }
    }
}

/// Compute storage with blob/share separation for DA
///
/// - Includes blocks, DA shares, and commitments
/// - Returns summaries, breakdowns, and recommendations
fn calculate_storage_requirements(config: &CalculationConfig) -> StorageCalculationResults {
    let blocks_per_day = (24 * 60 * 60) / config.network.block_time_seconds;
    let total_blocks_for_period = config.time_period.days * blocks_per_day;

    let block_data_for_period_gb = (total_blocks_for_period as f64
        * config.network.avg_block_size_bytes as f64)
        / (1024.0 * 1024.0 * 1024.0);

    let block_data = BlockDataResults {
        blocks_per_day,
        blocks_for_period: total_blocks_for_period,
        avg_block_size_kb: config.network.avg_block_size_bytes / 1024,
        total_block_data_gb: block_data_for_period_gb,
        time_period_description: config.time_period.description.clone(),
    };

    let mut scenarios = Vec::new();
    let mut scenario_storage_map = HashMap::new();

    for scenario in &config.scenarios {
        let typical_subnets_per_validator =
            config.network.total_subnets / (scenario.total_validators / 10).max(1);
        let subnet_assignment_probability =
            typical_subnets_per_validator as f64 / config.network.total_subnets as f64;

        let total_blobs_for_period = total_blocks_for_period * scenario.blobs_per_block;

        let validator_assigned_blobs =
            (total_blobs_for_period as f64 * subnet_assignment_probability) as u64;

        let shares_per_assigned_blob =
            config.network.shares_per_blob / config.network.total_subnets;
        let total_shares_stored = validator_assigned_blobs * shares_per_assigned_blob.max(1);
        let da_shares_size_gb = (total_shares_stored * config.network.da_share_size_bytes) as f64
            / (1024.0 * 1024.0 * 1024.0);

        let da_commitments_size_gb = (validator_assigned_blobs
            * config.network.da_commitment_size_bytes) as f64
            / (1024.0 * 1024.0 * 1024.0);

        let total_da_size_gb = da_shares_size_gb + da_commitments_size_gb;
        let total_storage_for_period = block_data_for_period_gb + total_da_size_gb;

        scenario_storage_map.insert(scenario.name.clone(), total_da_size_gb);

        scenarios.push(ScenarioResults {
            scenario_name: scenario.name.clone(),
            blobs_per_block: scenario.blobs_per_block,
            total_validators: scenario.total_validators,
            typical_subnets_per_validator,
            subnet_assignment_percent: subnet_assignment_probability * 100.0,
            shares_stored_count: total_shares_stored,
            blobs_assigned_count: validator_assigned_blobs,
            da_shares_gb: da_shares_size_gb,
            da_commitments_gb: da_commitments_size_gb,
            total_da_gb: total_da_size_gb,
            total_validator_storage_gb: total_storage_for_period,
        });
    }

    let breakdown_periods = if config.time_period.days >= 365 {
        12
    } else if config.time_period.days >= 30 {
        config.time_period.days / 7
    } else {
        config.time_period.days
    };

    let first_scenario_da_gb = scenario_storage_map.values().next().copied().unwrap_or(0.0);
    let total_gb_per_period = block_data_for_period_gb + first_scenario_da_gb;
    let increment_gb = total_gb_per_period / breakdown_periods as f64;

    let mut time_breakdown = Vec::new();
    for period in 1..=breakdown_periods {
        let cumulative_gb = increment_gb * period as f64;
        let period_desc = if config.time_period.days >= 365 {
            format!("Month {}", period)
        } else if config.time_period.days >= 30 {
            format!("Week {}", period)
        } else {
            format!("Day {}", period)
        };

        time_breakdown.push(TimeBreakdown {
            period_number: period,
            period_description: period_desc,
            cumulative_gb,
        });
    }

    let mut hardware_recommendations = Vec::new();
    for scenario in &scenarios {
        let storage_gb = scenario.total_validator_storage_gb as u64;
        let recommended = if storage_gb < 50 {
            "100GB+ storage"
        } else if storage_gb < 100 {
            "200GB+ storage"
        } else if storage_gb < 200 {
            "500GB+ storage"
        } else if storage_gb < 500 {
            "1TB+ storage"
        } else {
            "2TB+ storage"
        };

        hardware_recommendations.push(HardwareRecommendation {
            scenario: scenario.scenario_name.clone(),
            storage_gb_for_period: storage_gb,
            recommended_storage: recommended.to_string(),
        });
    }

    let stress_testing_notes = vec![
        "Memory pressure increases with database size".to_string(),
        "Cache efficiency decreases as dataset grows beyond memory".to_string(),
        "Compaction overhead increases with write frequency".to_string(),
        "Range scan performance degrades with database size".to_string(),
        "Storage benchmarks should test multi-GB datasets for realism".to_string(),
        format!(
            "Test with datasets representing {}-{} days of operation",
            config.time_period.days / 4,
            config.time_period.days / 2
        ),
    ];

    StorageCalculationResults {
        calculation_config: config.clone(),
        block_data,
        scenarios,
        time_breakdown,
        hardware_recommendations,
        stress_testing_notes,
    }
}

fn main() {
    let default_config = CalculationConfig::default();

    let monthly_config = CalculationConfig {
        time_period: TimePeriod {
            days: 30,
            description: "30 days".to_string(),
        },
        network: NetworkConfig::default(),
        scenarios: vec![
            NetworkScenario {
                name: "Testnet Conservative".to_string(),
                blobs_per_block: 25,
                total_validators: 100,
            },
            NetworkScenario {
                name: "Testnet Active".to_string(),
                blobs_per_block: 50,
                total_validators: 100,
            },
        ],
    };

    let weekly_config = CalculationConfig {
        time_period: TimePeriod {
            days: 7,
            description: "1 week".to_string(),
        },
        network: NetworkConfig {
            block_time_seconds: 15,
            shares_per_blob: 256,
            ..NetworkConfig::default()
        },
        scenarios: vec![NetworkScenario {
            name: "Development".to_string(),
            blobs_per_block: 10,
            total_validators: 10,
        }],
    };

    let configs = vec![
        ("annual", default_config),
        ("monthly", monthly_config),
        ("weekly", weekly_config),
    ];

    let mut all_results = HashMap::new();

    for (name, config) in configs {
        let results = calculate_storage_requirements(&config);
        all_results.insert(name, results);
    }

    save_capacity_results(&all_results);

    match serde_json::to_string_pretty(&all_results) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing results: {}", e),
    }
}

fn save_capacity_results(all_results: &HashMap<&str, StorageCalculationResults>) {
    let results_dir = BenchConfig::results_path();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("storage_capacity_calculation_{}.json", timestamp);
    let filepath = results_dir.join(filename);

    let calculation_results: std::collections::HashMap<String, StorageCalculationResults> =
        all_results
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();

    let report = CapacityCalculationReport {
        calculation_results,
        summary: CalculationSummary {
            scenarios_calculated: all_results.len(),
            total_time_periods: all_results
                .values()
                .map(|r| r.scenarios.len())
                .sum::<usize>(),
            calculation_timestamp: chrono::Utc::now().to_rfc3339(),
        },
        metadata: ReportMetadata {
            tool: "storage_capacity_calculator".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Storage capacity estimates for Nomos validator scenarios".to_string(),
        },
    };

    match fs::write(&filepath, serde_json::to_string_pretty(&report).unwrap()) {
        Ok(_) => eprintln!(
            "Capacity calculation results saved to: {}",
            filepath.display()
        ),
        Err(e) => eprintln!(
            "Failed to save capacity results to {}: {}",
            filepath.display(),
            e
        ),
    }
}
