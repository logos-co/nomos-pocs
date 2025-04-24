use nomos::{CryptarchiaInfo, HeaderId};
use reqwest::Url;
use tracing::{error, info};

pub const CRYPTARCHIA_INFO: &str = "cryptarchia/info";
pub const STORAGE_BLOCK: &str = "storage/block";

pub mod nomos;

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

    pub async fn get_cryptarchia_info(&self) -> Result<CryptarchiaInfo, String> {
        let url = self.base_url.join(CRYPTARCHIA_INFO).expect("Invalid URL");

        info!("Requesting cryptarchia info from {}", url);
        let request = self.reqwest_client.get(url).basic_auth(
            &self.basic_auth.username,
            self.basic_auth.password.as_deref(),
        );

        let response = request.send().await.map_err(|e| {
            error!("Failed to send request: {}", e);
            "Failed to send request".to_string()
        })?;

        if !response.status().is_success() {
            error!("Failed to get cryptarchia info: {}", response.status());
            return Err("Failed to get cryptarchia info".to_string());
        }

        let info = response.json::<CryptarchiaInfo>().await.map_err(|e| {
            error!("Failed to parse response: {}", e);
            "Failed to parse response".to_string()
        })?;
        Ok(info)
    }

    pub async fn get_block(&self, id: HeaderId) -> Result<serde_json::Value, String> {
        let url = self.base_url.join(STORAGE_BLOCK).expect("Invalid URL");

        info!("Requesting block with HeaderId {}", id);
        let request = self
            .reqwest_client
            .post(url)
            .header("Content-Type", "application/json")
            .basic_auth(
                &self.basic_auth.username,
                self.basic_auth.password.as_deref(),
            )
            .body(serde_json::to_string(&id).unwrap());

        let response = request.send().await.map_err(|e| {
            error!("Failed to send request: {}", e);
            "Failed to send request".to_string()
        })?;

        if !response.status().is_success() {
            error!("Failed to get block: {}", response.status());
            return Err("Failed to get block".to_string());
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse JSON: {}", e);
            "Failed to parse JSON".to_string()
        })?;

        info!(
            "Block (raw): {}",
            serde_json::to_string_pretty(&json).unwrap()
        );

        Ok(json)
    }
}
