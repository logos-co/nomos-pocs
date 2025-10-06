use bytes::Bytes;
use nomos_core::{da::BlobId, header::HeaderId};
use rand::Rng as _;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

use crate::{
    benchmark::utilities::{create_blob_id, create_header_id},
    deterministic::BenchmarkSeed,
};

pub struct RealisticDataGenerator {
    seed_config: BenchmarkSeed,
    dataset_rng: ChaCha20Rng,
    block_sequence: u64,
    da_sequence: u64,
    generation_stats: DataGenerationStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataGenerationStats {
    pub blocks_created: u64,
    pub da_shares_created: u64,
    pub commitments_created: u64,
    pub total_bytes_generated: u64,
    pub generation_start: Option<chrono::DateTime<chrono::Utc>>,
}

impl RealisticDataGenerator {
    #[must_use]
    pub fn new(master_seed: u64) -> Self {
        let seed_config = BenchmarkSeed::from_master(master_seed);
        let dataset_rng = seed_config.dataset_rng();

        Self {
            seed_config,
            dataset_rng,
            block_sequence: 0,
            da_sequence: 0,
            generation_stats: DataGenerationStats {
                generation_start: Some(chrono::Utc::now()),
                ..Default::default()
            },
        }
    }

    #[must_use]
    pub fn with_default_seed() -> Self {
        Self::new(12345)
    }

    pub fn generate_block(&mut self, target_size: usize) -> Bytes {
        let block_data = self.create_realistic_block_data(self.block_sequence, target_size);

        self.block_sequence += 1;
        self.generation_stats.blocks_created += 1;
        self.generation_stats.total_bytes_generated += target_size as u64;

        block_data
    }

    pub fn generate_da_share(&mut self, size: usize) -> Bytes {
        let share_data = self.create_deterministic_da_share(self.da_sequence, size);

        self.da_sequence += 1;
        self.generation_stats.da_shares_created += 1;
        self.generation_stats.total_bytes_generated += size as u64;

        share_data
    }

    pub fn generate_commitment(&mut self, size: usize) -> Bytes {
        let commitment_data = self.create_deterministic_commitment(self.da_sequence, size);

        self.generation_stats.commitments_created += 1;
        self.generation_stats.total_bytes_generated += size as u64;

        commitment_data
    }

    pub fn generate_block_batch(&mut self, count: usize, block_size: usize) -> Vec<Bytes> {
        std::iter::repeat_with(|| self.generate_block(block_size))
            .take(count)
            .collect()
    }

    pub fn generate_da_batch(
        &mut self,
        count: usize,
        share_size: usize,
        commitment_size: usize,
    ) -> Vec<(Bytes, Bytes)> {
        std::iter::repeat_with(|| {
            let share = self.generate_da_share(share_size);
            let commitment = self.generate_commitment(commitment_size);
            (share, commitment)
        })
        .take(count)
        .collect()
    }

    #[must_use]
    pub const fn stats(&self) -> &DataGenerationStats {
        &self.generation_stats
    }

    #[must_use]
    pub const fn sequence_state(&self) -> (u64, u64) {
        (self.block_sequence, self.da_sequence)
    }

    pub const fn set_sequence_state(&mut self, block_sequence: u64, da_sequence: u64) {
        self.block_sequence = block_sequence;
        self.da_sequence = da_sequence;
    }

    pub fn reset(&mut self) {
        self.block_sequence = 0;
        self.da_sequence = 0;
        self.generation_stats = DataGenerationStats {
            generation_start: Some(chrono::Utc::now()),
            ..Default::default()
        };
        self.dataset_rng = self.seed_config.dataset_rng();
    }

    fn create_realistic_block_data(&mut self, block_index: u64, target_size: usize) -> Bytes {
        let mut block_data = Vec::with_capacity(target_size);

        block_data.extend_from_slice(&block_index.to_be_bytes());

        let parent_hash: [u8; 32] = self.dataset_rng.gen();
        block_data.extend_from_slice(&parent_hash);

        let merkle_root: [u8; 32] = self.dataset_rng.gen();
        block_data.extend_from_slice(&merkle_root);

        let timestamp = chrono::Utc::now().timestamp() as u64 + block_index * 30;
        block_data.extend_from_slice(&timestamp.to_be_bytes());

        while block_data.len() < target_size {
            block_data.push(self.dataset_rng.gen());
        }

        block_data.resize(target_size, 0);
        Bytes::from(block_data)
    }

    fn create_deterministic_da_share(&mut self, _sequence: u64, size: usize) -> Bytes {
        let data: Vec<u8> = std::iter::repeat_with(|| self.dataset_rng.gen())
            .take(size)
            .collect();
        Bytes::from(data)
    }

    fn create_deterministic_commitment(&mut self, _sequence: u64, size: usize) -> Bytes {
        let data: Vec<u8> = std::iter::repeat_with(|| self.dataset_rng.gen())
            .take(size)
            .collect();
        Bytes::from(data)
    }
}

pub struct IdGenerator {
    block_counter: usize,
    blob_counter: usize,
}

impl IdGenerator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            block_counter: 0,
            blob_counter: 0,
        }
    }

    pub fn next_header_id(&mut self) -> HeaderId {
        let id = create_header_id(self.block_counter);
        self.block_counter += 1;
        id
    }

    pub fn next_blob_id(&mut self) -> BlobId {
        let id = create_blob_id(self.blob_counter, 0);
        self.blob_counter += 1;
        id
    }

    #[must_use]
    pub const fn counters(&self) -> (usize, usize) {
        (self.block_counter, self.blob_counter)
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
