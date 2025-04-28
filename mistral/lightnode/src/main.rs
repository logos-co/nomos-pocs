use clap::Parser;
use evm_lightnode::{Credentials, NomosClient, nomos::HeaderId, proofcheck};
use futures::StreamExt;
use std::error;
use std::path::PathBuf;
use tracing::info;
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

const TESTNET_EXECUTOR: &str = "https://testnet.nomos.tech/node/3/";

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

    //todo: use check_da_periodically to check for new blocks from da

    Ok(())
}

#[allow(dead_code)]
async fn check_da_periodically() -> Result<(), Box<dyn error::Error>> {
    let url = std::env::var("NOMOS_EXECUTOR").unwrap_or(TESTNET_EXECUTOR.to_string());
    let user = std::env::var("NOMOS_USER").unwrap_or_default();
    let password = std::env::var("NOMOS_PASSWORD").unwrap_or_default();
    let basic_auth = Credentials {
        username: user,
        password: Some(password),
    };

    let nomos_client = NomosClient::new(Url::parse(&url).unwrap(), basic_auth);

    let mut current_tip = HeaderId::default();

    loop {
        let info = nomos_client.get_cryptarchia_info().await?;

        if info.tip != current_tip {
            current_tip = info.tip;
            info!("New tip: {:?}", current_tip);
            let block = nomos_client.get_block(info.tip).await?;
            info!("Block: {:?}", block);

            let blobs = block.get("bl_blobs");

            if blobs.is_none() {
                info!("Parsing error");
                continue;
            };

            let blobs = blobs.unwrap().as_array().unwrap();
            if blobs.is_empty() {
                info!("No blobs found in block");
                continue;
            }
            for blob in blobs {
                let id_array = blob.get("id").unwrap().as_array().unwrap();

                let mut blob_id = [0u8; 32];
                for (i, num) in id_array.iter().enumerate() {
                    if i < 32 {
                        blob_id[i] = num.as_u64().unwrap() as u8;
                    }
                }

                info!("fetching stream");
                let shares_stream = nomos_client.get_shares(blob_id).await?;

                let shares = shares_stream.collect::<Vec<_>>().await;

                info!("Shares for blob_id {:?}: {:?}", blob_id, shares);

                // todo: verify proof
            }
        } else {
            info!("No new tip, sleeping...");
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}
