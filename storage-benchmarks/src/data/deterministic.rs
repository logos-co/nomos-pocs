use rand::{Rng as _, SeedableRng as _};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSeed {
    pub master_seed: u64,
    pub dataset_generation_seed: u64,
    pub access_pattern_seed: u64,
    pub latency_measurement_seed: u64,
}

impl BenchmarkSeed {
    #[must_use]
    pub fn from_master(master_seed: u64) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(master_seed);

        Self {
            master_seed,
            dataset_generation_seed: rng.gen(),
            access_pattern_seed: rng.gen(),
            latency_measurement_seed: rng.gen(),
        }
    }

    #[must_use]
    pub fn default_fixed() -> Self {
        Self::from_master(12345)
    }

    #[must_use]
    pub fn from_args_or_env(args: &[String]) -> Self {
        for (i, arg) in args.iter().enumerate() {
            if arg == "--seed" && i + 1 < args.len() {
                if let Ok(seed) = args[i + 1].parse::<u64>() {
                    return Self::from_master(seed);
                }
            }
        }

        if let Ok(seed_str) = std::env::var("BENCHMARK_SEED") {
            if let Ok(seed) = seed_str.parse::<u64>() {
                return Self::from_master(seed);
            }
        }

        Self::default_fixed()
    }

    #[must_use]
    pub fn dataset_rng(&self) -> ChaCha20Rng {
        ChaCha20Rng::seed_from_u64(self.dataset_generation_seed)
    }

    #[must_use]
    pub fn access_pattern_rng(&self, operation_id: u64) -> ChaCha20Rng {
        ChaCha20Rng::seed_from_u64(self.access_pattern_seed.wrapping_add(operation_id))
    }

    #[must_use]
    pub fn latency_measurement_rng(&self) -> ChaCha20Rng {
        ChaCha20Rng::seed_from_u64(self.latency_measurement_seed)
    }

    pub fn log_configuration(&self) {
        log::info!("Benchmark seeds (for reproducibility):");
        log::info!("   Master seed: {}", self.master_seed);
        log::info!("   Dataset generation: {}", self.dataset_generation_seed);
        log::info!("   Access patterns: {}", self.access_pattern_seed);
        log::info!("   Latency measurement: {}", self.latency_measurement_seed);
        log::info!(
            "   Reproduce with: --seed {} or BENCHMARK_SEED={}",
            self.master_seed,
            self.master_seed
        );
    }
}

static GLOBAL_BENCHMARK_SEED: std::sync::OnceLock<BenchmarkSeed> = std::sync::OnceLock::new();

pub fn initialize_benchmark_seed(args: &[String]) -> &'static BenchmarkSeed {
    GLOBAL_BENCHMARK_SEED.get_or_init(|| {
        let seed = BenchmarkSeed::from_args_or_env(args);
        seed.log_configuration();
        seed
    })
}

pub fn get_benchmark_seed() -> &'static BenchmarkSeed {
    GLOBAL_BENCHMARK_SEED.get().unwrap_or_else(|| {
        GLOBAL_BENCHMARK_SEED.get_or_init(|| {
            let seed = BenchmarkSeed::default_fixed();
            log::warn!("Using default seed (benchmark_seed not initialized)");
            seed.log_configuration();
            seed
        })
    })
}

#[must_use]
pub fn create_deterministic_rng(purpose: RngPurpose, id: u64) -> ChaCha20Rng {
    let seed = get_benchmark_seed();

    match purpose {
        RngPurpose::DatasetGeneration => {
            ChaCha20Rng::seed_from_u64(seed.dataset_generation_seed.wrapping_add(id))
        }
        RngPurpose::AccessPattern => seed.access_pattern_rng(id),
        RngPurpose::LatencyMeasurement => {
            ChaCha20Rng::seed_from_u64(seed.latency_measurement_seed.wrapping_add(id))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RngPurpose {
    DatasetGeneration,
    AccessPattern,
    LatencyMeasurement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_seed_derivation() {
        let seed1 = BenchmarkSeed::from_master(12345);
        let seed2 = BenchmarkSeed::from_master(12345);

        assert_eq!(seed1.dataset_generation_seed, seed2.dataset_generation_seed);
        assert_eq!(seed1.access_pattern_seed, seed2.access_pattern_seed);
    }

    #[test]
    fn test_different_master_seeds() {
        let seed1 = BenchmarkSeed::from_master(12345);
        let seed2 = BenchmarkSeed::from_master(54321);

        assert_ne!(seed1.dataset_generation_seed, seed2.dataset_generation_seed);
    }

    #[test]
    fn test_deterministic_rng_creation() {
        let seed = BenchmarkSeed::from_master(12345);

        let rng1 = seed.access_pattern_rng(100);
        let rng2 = seed.access_pattern_rng(100);

        assert_eq!(rng1.get_seed(), rng2.get_seed());
    }
}
