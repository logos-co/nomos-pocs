use tokio::process::Command;
use std::path::Path;
use std::io::Write;
use reqwest::Url;
use tracing::{error, info};

pub async fn verify_proof(
    block_number: u64,
    block_count: u64,
    rpc: &Url,
    prover_url: &Url,
    zeth_bin: &Path,
) -> Result<(), String> {
    info!(
        "Verifying proof for blocks {}-{}",
        block_number,
        block_number + block_count - 1
    );

    let url = prover_url.join(&format!(
        "/?block_start={}&block_count={}",
        block_number, block_count
    )).map_err(|e| format!("Failed to construct URL: {}", e))?;
    let proof = reqwest::get(url).await
        .map_err(|e| format!("Failed to fetch proof: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read proof response: {}", e))?;

    let mut tempfile = tempfile::NamedTempFile::new()
        .map_err(|e| format!("Failed to create temporary file: {}", e))?;
    tempfile.write_all(&proof)
        .map_err(|e| format!("Failed to write proof to file: {}", e))?;
  

    let output = Command::new(zeth_bin)
        .args([
            "verify",
            &format!("--rpc={}", rpc),
            &format!("--block-number={}", block_number),
            &format!("--block-count={}", block_count),
            &format!("--file={}", tempfile.path().display()),
        ])
        .output().await
        .map_err(|e| format!("Failed to execute zeth-ethereum verify: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("zeth-ethereum verify command failed: {}", stderr);
        return Err(format!(
            "zeth-ethereum verify command failed with status: {}\nStderr: {}",
            output.status, stderr
        ));
    }

    info!("Proof verification successful");

    Ok(())
}
