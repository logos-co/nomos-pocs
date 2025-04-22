use clap::Parser;
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Ethereum Proof Generation Tool")]
struct Args {
    #[clap(long, default_value = "http://localhost:8546")]
    rpc: String,

    #[clap(long, default_value = "5")]
    block_number: u64,

    #[clap(long, default_value = "10")]
    batch_size: u64,
}

fn run_ethereum_prove(rpc: &str, block_number: u64, batch_size: u64) -> Result<(), String> {
    println!(
        "Running Ethereum prove for blocks {}-{}",
        block_number,
        block_number + batch_size - 1
    );

    let output = Command::new("just")
        .args([
            "ethereum",
            "prove",
            &format!("--rpc={}", rpc),
            &format!("--block-number={}", block_number),
            &format!("--block-count={}", batch_size),
        ])
        .output()
        .map_err(|e| format!("Failed to execute just ethereum prove: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ethereum prove command failed with status: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("Successfully processed batch");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Starting Ethereum prover...");
    run_ethereum_prove(&args.rpc, args.block_number, args.batch_size)?;

    Ok(())
}
