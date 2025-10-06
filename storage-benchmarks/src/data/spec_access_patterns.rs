use rand_distr::{Distribution as _, Zipf};

use crate::{
    config::ValidatorProfile,
    deterministic::{create_deterministic_rng, RngPurpose},
};

#[must_use]
pub fn select_block_spec_accurate(
    operation_count: u64,
    max_blocks: usize,
    profile: &ValidatorProfile,
) -> usize {
    if max_blocks == 0 {
        return 0;
    }

    let access_selector = (operation_count * 31) % 100;

    if access_selector < (profile.recent_access_ratio * 100.0) as u64 {
        select_recent_block_zipfian(operation_count, max_blocks)
    } else {
        select_historical_block_uniform(operation_count, max_blocks)
    }
}

fn select_recent_block_zipfian(operation_count: u64, max_blocks: usize) -> usize {
    let recent_window_size = std::cmp::max(max_blocks / 5, 1000);

    let zipf_dist = Zipf::new(recent_window_size as u64, 1.0).unwrap();

    let mut rng = create_deterministic_rng(RngPurpose::AccessPattern, operation_count);
    let zipf_sample = zipf_dist.sample(&mut rng) as usize;

    let recent_start = max_blocks.saturating_sub(recent_window_size);
    let tip_offset = zipf_sample.saturating_sub(1);

    recent_start + (recent_window_size - 1 - tip_offset)
}

const fn select_historical_block_uniform(operation_count: u64, max_blocks: usize) -> usize {
    (operation_count as usize * 23) % max_blocks
}

#[must_use]
pub fn select_da_spec_accurate(
    operation_count: u64,
    max_blobs: usize,
    profile: &ValidatorProfile,
) -> usize {
    if max_blobs == 0 {
        return 0;
    }

    let recent_threshold = (profile.recent_access_ratio * 100.0) as u64;
    let access_selector = (operation_count * 41) % 100;

    if access_selector < recent_threshold {
        let recent_blobs = std::cmp::min(100, max_blobs);
        let zipf_dist = Zipf::new(recent_blobs as u64, 1.2).unwrap();
        let mut rng = create_deterministic_rng(RngPurpose::AccessPattern, operation_count);
        let sample = zipf_dist.sample(&mut rng) as usize;

        let recent_start = max_blobs.saturating_sub(recent_blobs);
        recent_start + (recent_blobs - sample.min(recent_blobs))
    } else {
        (operation_count as usize * 29) % max_blobs
    }
}
