use std::time::Duration;

use crate::config::ValidatorProfile;

#[must_use]
pub fn create_header_id(index: usize) -> nomos_core::header::HeaderId {
    let mut id = [0u8; 32];
    id[0..4].copy_from_slice(&(index as u32).to_be_bytes());
    nomos_core::header::HeaderId::from(id)
}

#[must_use]
pub fn create_blob_id(block: usize, blob_in_block: usize) -> nomos_core::da::BlobId {
    let mut id = [0u8; 32];
    id[0..4].copy_from_slice(&(block as u32).to_be_bytes());
    id[4..8].copy_from_slice(&(blob_in_block as u32).to_be_bytes());
    nomos_core::da::BlobId::from(id)
}

pub fn safe_interval_from_hz(frequency_hz: f64, workload_type: &str) -> Result<Duration, String> {
    if frequency_hz <= 0.0 {
        return Err(format!(
            "Invalid frequency {frequency_hz} Hz for {workload_type}"
        ));
    }

    let interval_ms = (1000.0 / frequency_hz) as u64;
    Ok(Duration::from_millis(interval_ms))
}

#[must_use]
pub fn estimate_sequential_performance(profile: &ValidatorProfile) -> f64 {
    profile.range_scan_rate_hz.mul_add(
        10.0,
        profile.block_read_rate_hz + profile.da_share_read_rate_hz + profile.da_share_write_rate_hz,
    )
}
