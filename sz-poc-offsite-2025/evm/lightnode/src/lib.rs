use std::ops::Range;

use executor_http_client::{BasicAuthCredentials, ExecutorHttpClient};
use kzgrs_backend::common::share::DaShare;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::{error, error_span, info};

pub const DA_GET_RANGE: &str = "/da/get-range";

pub struct NomosDa {
    url: Url,
    client: ExecutorHttpClient,
    reqwest_client: reqwest::Client,
}

impl NomosDa {
    pub fn new(basic_auth: BasicAuthCredentials, url: Url) -> Self {
        Self {
            client: ExecutorHttpClient::new(Some(basic_auth)),
            url,
            reqwest_client: reqwest::Client::new(),
        }
    }

    pub async fn get_indexer_range(
        &self,
        app_id: [u8; 32],
        range: Range<[u8; 8]>,
    ) -> Vec<([u8; 8], Vec<DaShare>)> {
        let endpoint = self
            .url
            .join(DA_GET_RANGE)
            .expect("Failed to construct valid URL");

        match self
            .reqwest_client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&GetRangeReq { app_id, range }).unwrap())
            .send()
            .await
            .unwrap()
            .json::<Vec<([u8; 8], Vec<DaShare>)>>()
            .await
        {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to get indexer range: {e}");
                vec![]
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GetRangeReq {
    pub app_id: [u8; 32],
    pub range: Range<[u8; 8]>,
}
