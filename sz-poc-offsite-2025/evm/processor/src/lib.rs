use reth_ethereum::Block;
use executor_http_client::{ExecutorHttpClient, Error};
pub use executor_http_client::BasicAuthCredentials;
use reqwest::Url;
use reth_tracing::tracing::{info, error};
use kzgrs_backend::{dispersal::Metadata, encoder::DaEncoderParams};
// TODO: The logic to batch multiple of these blocks (or the transactions within them) and send them to DA and generate proofs is still missing. It will have to be added at the offsite.
// This type does not support any recovery mechanism, so if the node is stopped, the state DB should be cleaned before starting again. The folder is specified by the `--datadir` option in the binary.
pub struct Processor {
    da: NomosDa,
}

impl Processor {
    pub fn new(da: NomosDa) -> Self {
        Self {
            da
        }
    }

    pub async fn process_blocks(&mut self, new_blocks: impl Iterator<Item = Block>) {
        for block in new_blocks {
            let mut blob = bincode::serialize(&block).expect("Failed to serialize block");
            let metadata = Metadata::new([0; 32], block.number.into());
            // the node expects blobs to be padded to the next chunk size
            let remainder = blob.len() % DaEncoderParams::MAX_BLS12_381_ENCODING_CHUNK_SIZE;
            blob.extend(
                std::iter::repeat(0)
                    .take(DaEncoderParams::MAX_BLS12_381_ENCODING_CHUNK_SIZE - remainder),
            );
            if let Err(e) = self.da.disperse(blob, metadata).await 
            {
                error!("Failed to disperse block: {e}");
            } else {
                info!("Dispersed block: {:?}", block);
            }
        }
    }
}



pub struct NomosDa {
    url: Url,
    client: ExecutorHttpClient,
}



impl NomosDa {
    pub fn new(basic_auth: BasicAuthCredentials, url: Url) -> Self {
        Self {
            client: ExecutorHttpClient::new(Some(basic_auth)),
            url,
        }
    }   

    pub async fn disperse(&self, data: Vec<u8>, metadata: Metadata) -> Result<(), Error> {
        self.client
            .publish_blob(self.url.clone(), data, metadata).await
            
    }
}