use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetGenConfig {
    pub dataset: DatasetParams,
    pub network: NetworkParams,
    pub validator: ValidatorParams,
    pub blocks: BlockParams,
    pub da: DataAvailabilityParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetParams {
    pub days: usize,
    pub block_time_seconds: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkParams {
    pub load_name: String,
    pub blobs_per_block: usize,
    pub total_subnets: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorParams {
    pub assigned_subnets: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockParams {
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAvailabilityParams {
    pub share_size_bytes: usize,
    pub commitment_size_bytes: usize,
    pub shares_per_blob: usize,
}

impl DatasetGenConfig {
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self =
            toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {e}"))?;
        Ok(config)
    }

    #[must_use]
    pub const fn total_blocks(&self) -> usize {
        let blocks_per_day = (24 * 60 * 60) / self.dataset.block_time_seconds as usize;
        self.dataset.days * blocks_per_day
    }

    #[must_use]
    pub fn estimated_size(&self) -> String {
        let total_blocks = self.total_blocks() as u64;
        let block_size = self.blocks.size_bytes as u64;

        let subnet_assignment_probability =
            self.validator.assigned_subnets as f64 / self.network.total_subnets as f64;

        let total_blobs = total_blocks * self.network.blobs_per_block as u64;
        let validator_assigned_blobs = (total_blobs as f64 * subnet_assignment_probability) as u64;

        let shares_per_assigned_blob =
            self.da.shares_per_blob as u64 / self.network.total_subnets as u64;
        let total_shares_stored = validator_assigned_blobs * shares_per_assigned_blob;

        let block_data_size = total_blocks * block_size;
        let da_shares_size = total_shares_stored * self.da.share_size_bytes as u64;
        let da_commitments_size = validator_assigned_blobs * self.da.commitment_size_bytes as u64;
        let da_data_size = da_shares_size + da_commitments_size;
        let total_bytes = block_data_size + da_data_size;

        if total_bytes < 1024 * 1024 {
            format!("{:.1} KB", total_bytes as f64 / 1024.0)
        } else if total_bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", total_bytes as f64 / 1024.0 / 1024.0)
        } else {
            format!("{:.1} GB", total_bytes as f64 / 1024.0 / 1024.0 / 1024.0)
        }
    }
}
