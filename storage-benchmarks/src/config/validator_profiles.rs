use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorProfile {
    pub name: String,
    pub description: String,

    pub block_read_rate_hz: f64,
    pub da_share_read_rate_hz: f64,
    pub range_scan_rate_hz: f64,

    pub block_write_rate_hz: f64,
    pub da_share_write_rate_hz: f64,
    pub commitment_write_rate_hz: f64,

    pub recent_access_ratio: f64,
    pub historical_access_ratio: f64,

    #[serde(default = "default_total_validators")]
    pub total_validators: usize,
    #[serde(default = "default_assigned_subnets")]
    pub assigned_subnets: usize,
}

impl ValidatorProfile {
    #[must_use]
    pub fn ibd_concurrent_streams(&self) -> usize {
        let base_streams = 1;
        let network_factor = (self.total_validators as f64 / 500.0).max(1.0);
        let total_streams = (f64::from(base_streams) * network_factor).round() as usize;

        std::cmp::min(total_streams, 8)
    }

    #[must_use]
    pub fn da_concurrent_streams(&self) -> usize {
        let subnet_factor = (self.assigned_subnets as f64 / 5.0).max(1.0);
        let total_streams = subnet_factor.round() as usize;

        std::cmp::min(total_streams, 5)
    }

    #[must_use]
    pub fn total_concurrent_services(&self) -> usize {
        let base_services = 3;
        let ibd_services = self.ibd_concurrent_streams();
        let da_services = self.da_concurrent_streams();

        base_services + ibd_services + da_services
    }
}

const fn default_total_validators() -> usize {
    1000
}
const fn default_assigned_subnets() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkScalingConfig {
    pub total_validators: usize,

    pub total_subnets: usize,

    pub assigned_subnets: usize,

    pub activity_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    pub base_concurrent_services: usize,

    pub services_per_1k_validators: f64,

    pub max_concurrent_services: usize,

    pub ibd_concurrency_factor: f64,

    pub da_concurrency_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorProfiles {
    pub light: ValidatorProfile,
    pub mainnet: ValidatorProfile,
    pub testnet: ValidatorProfile,
}

impl ValidatorProfiles {
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let profiles: Self = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse validator profiles TOML: {e}"))?;
        Ok(profiles)
    }

    #[must_use]
    pub fn get_profile(&self, name: &str) -> Option<&ValidatorProfile> {
        match name {
            "light" => Some(&self.light),
            "mainnet" => Some(&self.mainnet),
            "testnet" => Some(&self.testnet),
            _ => None,
        }
    }

    #[must_use]
    pub fn available_profiles(&self) -> Vec<&str> {
        vec!["light", "mainnet", "testnet"]
    }
}
