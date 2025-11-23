use common::error::AppError;
use reqwest::{Client, multipart};
use serde::Deserialize;

#[derive(Deserialize)]
struct IpfsAddResponse {
    #[serde(rename = "Hash")]
    hash: String,
}

pub struct IpfsClient {
    base_url: String,
    client: Client,
}

impl IpfsClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    pub async fn add(&self, data: Vec<u8>) -> Result<String, AppError> {
        let part = multipart::Part::bytes(data).file_name("file");
        let form = multipart::Form::new().part("file", part);

        let res = self.client
            .post(format!("{}/api/v0/add", self.base_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Ipfs(e.to_string()))?;

        if !res.status().is_success() {
            return Err(AppError::Ipfs(format!("IPFS add failed: {}", res.status())));
        }

        let ipfs_res: IpfsAddResponse = res.json().await
            .map_err(|e| AppError::Ipfs(e.to_string()))?;

        Ok(ipfs_res.hash)
    }

    pub async fn cat(&self, cid: &str) -> Result<Vec<u8>, AppError> {
        let res = self.client
            .post(format!("{}/api/v0/cat?arg={}", self.base_url, cid))
            .send()
            .await
            .map_err(|e| AppError::Ipfs(e.to_string()))?;

        if !res.status().is_success() {
            return Err(AppError::Ipfs(format!("IPFS cat failed: {}", res.status())));
        }

        let bytes = res.bytes().await
            .map_err(|e| AppError::Ipfs(e.to_string()))?;

        Ok(bytes.to_vec())
    }
}
