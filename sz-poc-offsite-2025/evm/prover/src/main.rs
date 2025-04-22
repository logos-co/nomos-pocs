use clap::Parser;
use std::{process::Command, thread, time::Duration};

#[derive(Parser, Debug)]
#[clap(author, version, about = "Ethereum Proof Generation Tool")]
struct Args {
    #[clap(long, default_value = "http://localhost:8546")]
    rpc: String,

    #[clap(long, default_value = "5")]
    start_block: u64,

    #[clap(long, default_value = "10")]
    batch_size: u64,

    #[clap(long, default_value = "5")]
    interval: u64,
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

fn get_latest_block(rpc: &str) -> Result<u64, String> {
    println!("Checking latest block height...");

    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            rpc,
            "-d",
            r#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}"#,
        ])
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "curl command failed with status: {}",
            output.status
        ));
    }

    let response = String::from_utf8_lossy(&output.stdout);

    let block_hex = response
        .split("\"result\":\"")
        .nth(1)
        .ok_or("Failed to parse response")?
        .split("\"")
        .next()
        .ok_or("Failed to parse block number")?
        .trim_start_matches("0x");

    let block_number = u64::from_str_radix(block_hex, 16)
        .map_err(|e| format!("Failed to parse hex block number: {}", e))?;

    println!("Latest block: {}", block_number);
    Ok(block_number)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Starting Ethereum prover...");
    // todo: read current block from state maybe
    let mut current_block = args.start_block;
    loop {
        match get_latest_block(&args.rpc) {
            Ok(latest_block) => {
                if latest_block >= current_block + args.batch_size {
                    println!(
                        "New blocks available. Current: {}, Latest: {}",
                        current_block, latest_block
                    );

                    match run_ethereum_prove(&args.rpc, current_block, args.batch_size) {
                        Ok(_) => {
                            current_block += args.batch_size;
                            println!("Updated current block to {}", current_block);
                        }
                        Err(e) => {
                            eprintln!("Error running prover: {}", e);
                        }
                    }
                } else {
                    println!(
                        "No new blocks to process. Current: {}, Latest: {}",
                        current_block, latest_block
                    );
                }
            }
            Err(e) => {
                eprintln!("Error getting latest block: {}", e);
            }
        }

        println!("Sleeping for {} seconds...", args.interval);
        thread::sleep(Duration::from_secs(args.interval));
    }
}
