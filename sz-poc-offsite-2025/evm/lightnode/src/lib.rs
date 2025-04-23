use std::ops::Range;

use executor_http_client::{BasicAuthCredentials, ExecutorHttpClient};
use nomos::CryptarchiaInfo;
use reqwest::{RequestBuilder, Url};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

pub const CRYPTARCHIA_INFO: &str = "/cryptarchia/info";
pub const STORAGE_BLOCK: &str = "/storage/block";

mod nomos;

#[derive(Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: Option<String>,
}

pub struct NomosClient {
    base_url: Url,
    reqwest_client: reqwest::Client,
    basic_auth: Credentials,
}

impl NomosClient {
    pub fn new(base_url: Url, basic_auth: Credentials) -> Self {
        Self {
            base_url,
            reqwest_client: reqwest::Client::new(),
            basic_auth,
        }
    }

    pub async fn get_cryptarchia_info(&self) -> Result<CryptarchiaInfo, reqwest::Error> {
        let url = self.base_url.join(CRYPTARCHIA_INFO).expect("Invalid URL");

        let request = self.reqwest_client.get(url).basic_auth(
            self.basic_auth.username.clone(),
            self.basic_auth.password.clone(),
        );

        info!("Sending request with creds {:?}", self.basic_auth);

        let response = request.send().await?;
        if !response.status().is_success() {
            error!("Failed to get cryptarchia info: {}", response.status());
        }

        let info = response.json::<CryptarchiaInfo>().await?;
        Ok(info)
    }
}
