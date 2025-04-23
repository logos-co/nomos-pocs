use clap::Parser;
use evm_proof_checker::NomosDa;
use executor_http_client::BasicAuthCredentials;
use reqwest::Url;
use std::error;
use tracing_subscriber::{EnvFilter, fmt};

const TESTNET_EXECUTOR: &str = "https://testnet.nomos.tech/node/3/";

#[derive(Parser, Debug)]
#[clap(author, version, about = "Ethereum Proof Generation Tool")]
struct Args {
    // #[clap(long, required = true)]
    // block: u64,

    // #[clap(long, required = true)]
    // proof_file_path: String,

    // #[clap(long)]
    // zeth_binary_dir: Option<String>,
    #[clap(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    fmt::fmt().with_env_filter(filter).with_target(false).init();

    let url = std::env::var("NOMOS_EXECUTOR").unwrap_or(TESTNET_EXECUTOR.to_string());
    let user = std::env::var("NOMOS_USER").unwrap_or_default();
    let password = std::env::var("NOMOS_PASSWORD").unwrap_or_default();
    let da = NomosDa::new(
        BasicAuthCredentials::new(user, Some(password)),
        Url::parse(&url).unwrap(),
    );

    let from = 0u64.to_be_bytes();
    let to = 1u64.to_be_bytes();

    da.get_indexer_range([0; 32], from..to)
        .await
        .iter()
        .for_each(|(key, value)| {
            println!("Key: {:?}, Value: {:?}", key, value);
        });

    Ok(())
}
