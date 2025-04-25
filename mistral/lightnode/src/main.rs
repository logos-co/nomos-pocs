use clap::Parser;
use evm_lightnode::proofcheck;
use std::error;
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, fmt};
use url::Url;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Light Node validator")]
struct Args {
    #[clap(long, default_value = "info")]
    log_level: String,

    #[clap(long, default_value = "http://localhost:8546")]
    rpc: Url,

    #[clap(long, default_value = "http://localhost:8070")]
    prover_url: Url,

    #[clap(long)]
    start_block: u64,

    #[clap(long, default_value = "10")]
    batch_size: u64,

    #[clap(long)]
    zeth_binary_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    fmt::fmt().with_env_filter(filter).with_target(false).init();

    proofcheck::verify_proof(
        args.start_block,
        args.batch_size,
        &args.rpc,
        &args.prover_url,
        &args
            .zeth_binary_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("zeth-ethereum"),
    )
    .await?;

    Ok(())
}
