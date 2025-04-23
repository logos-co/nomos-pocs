use clap::Parser;
use evm_lightnode::{Credentials, NomosClient};
use executor_http_client::BasicAuthCredentials;
use url::Url;

use std::error;
use tracing_subscriber::{EnvFilter, fmt};

const TESTNET_EXECUTOR: &str = "https://testnet.nomos.tech/node/3/";

#[derive(Parser, Debug)]
#[clap(author, version, about = "Ethereum Proof Generation Tool")]
struct Args {
    #[clap(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let url = std::env::var("NOMOS_EXECUTOR").unwrap_or(TESTNET_EXECUTOR.to_string());
    let user = std::env::var("NOMOS_USER").unwrap_or_default();
    let password = std::env::var("NOMOS_PASSWORD").unwrap_or_default();
    let basic_auth = Credentials {
        username: user,
        password: Some(password),
    };

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    fmt::fmt().with_env_filter(filter).with_target(false).init();

    let consensus = NomosClient::new(Url::parse(&url).unwrap(), basic_auth);

    let info = consensus.get_cryptarchia_info().await?;
    println!("Cryptarchia Info: {:?}", info);

    Ok(())
}
