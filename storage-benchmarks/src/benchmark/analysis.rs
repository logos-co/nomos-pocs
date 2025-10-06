use nomos_storage::{
    api::{chain::StorageChainApi as _, da::StorageDaApi as _},
    backends::rocksdb::RocksBackend,
};

use super::{create_blob_id, create_header_id};

pub async fn analyze_dataset(
    storage: &mut RocksBackend,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    log::info!("Analyzing dataset size with adaptive probing...");

    let mut upper_bound = 10000;
    while upper_bound < 10_000_000 {
        let header_id = create_header_id(upper_bound);
        let block_result = storage.get_block(header_id).await;
        match block_result {
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => {
                break;
            }
        }
        upper_bound *= 2;
    }

    let mut low = upper_bound / 2;
    let mut high = upper_bound;
    let mut block_count = low;

    while low <= high {
        let mid = usize::midpoint(low, high);
        let header_id = create_header_id(mid);

        match storage.get_block(header_id).await {
            Ok(Some(_)) => {
                block_count = mid;
                low = mid + 1;
            }
            _ => {
                high = mid - 1;
            }
        }
    }

    let mut share_count = 0;
    let da_sample_size = std::cmp::min(1000, block_count / 100);

    for blob_idx in 0..da_sample_size {
        for subnet in 0..50 {
            let blob_id = create_blob_id(blob_idx, 0);
            let share_idx = [subnet as u8, 0u8];
            if let Ok(Some(_)) = storage.get_light_share(blob_id, share_idx).await {
                share_count += 1;
            }
        }
    }

    let estimated_da_total = if da_sample_size > 0 {
        share_count * (block_count / da_sample_size)
    } else {
        share_count
    };

    log::info!("DA estimation: sampled {share_count} objects from {da_sample_size} blocks, extrapolated to {estimated_da_total} total (assumes uniform distribution)");

    log::info!(
        "Dataset analysis complete: {block_count} blocks, ~{estimated_da_total} DA objects (sampled)"
    );

    Ok((block_count, estimated_da_total))
}
