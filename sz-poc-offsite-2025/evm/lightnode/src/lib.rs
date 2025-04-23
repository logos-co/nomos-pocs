use std::ops::Range;

use executor_http_client::{BasicAuthCredentials, ExecutorHttpClient};
use nomos::CryptarchiaInfo;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::{error, error_span, info};

pub const CRYPTARCHIA_INFO: &str = "/cryptarchia/info";
pub const STORAGE_BLOCK: &str = "/storage/block";

mod nomos;

pub struct NomosClient {
    base_url: Url,
    reqwest_client: reqwest::Client,
}

impl NomosClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            reqwest_client: reqwest::Client::new(),
        }
    }

    pub async fn get_cryptarchia_info(
        &self,
        base_url: Url,
    ) -> Result<CryptarchiaInfo, reqwest::Error> {
        let url = base_url.join(CRYPTARCHIA_INFO).expect("Invalid URL");
        let response = self.reqwest_client.get(url).send().await?;
        let info = response.json::<CryptarchiaInfo>().await?;
        Ok(info)
    }
}
