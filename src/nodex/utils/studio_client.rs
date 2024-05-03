use crate::{
    network_config,
    services::{internal::did_vc::DIDVCService, nodex::NodeX},
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Url,
};
use sha2::Sha256;

use crate::services::internal::didcomm_encrypted::DIDCommEncryptedService;
use serde_json::json;

type HmacSha256 = Hmac<Sha256>;

pub struct StudioClientConfig {
    pub base_url: String,
}

pub struct StudioClient {
    pub base_url: Url,
    pub instance: reqwest::Client,
    pub service: DIDCommEncryptedService,
}

impl StudioClient {
    pub fn new(_config: &StudioClientConfig) -> anyhow::Result<Self> {
        let url = Url::parse(&_config.base_url.to_string())?;
        let client: reqwest::Client = reqwest::Client::new();
        let service: DIDCommEncryptedService =
            DIDCommEncryptedService::new(NodeX::new(), DIDVCService::new(NodeX::new()));

        Ok(StudioClient {
            instance: client,
            base_url: url,
            service,
        })
    }

    fn auth_headers(&self, payload: String) -> anyhow::Result<HeaderMap> {
        let config = network_config();
        let secret = config.lock().get_secret_key().unwrap();
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;

        mac.update(payload.as_bytes());
        let signature = &hex::encode(mac.finalize().into_bytes());
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Nodex-Signature",
            HeaderValue::from_str(signature).unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        Ok(headers)
    }

    #[allow(dead_code)]
    pub async fn get(&self, _path: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(_path)?;
        let headers = self.auth_headers("".to_string())?;

        let response = self.instance.get(url).headers(headers).send().await?;
        Ok(response)
    }

    pub async fn post(&self, path: &str, body: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        let headers = self.auth_headers(body.to_string())?;

        let response = self
            .instance
            .post(url)
            .headers(headers)
            .body(body.to_string())
            .send()
            .await?;

        Ok(response)
    }

    pub async fn send_device_info(
        &self,
        path: &str,
        project_did: &str,
        mac_address: &str,
        version: &str,
        os: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let message = json!({
            "mac_address": mac_address,
            "version": version,
            "os": os,
        });
        let service = DIDCommEncryptedService::new(NodeX::new(), DIDVCService::new(NodeX::new()));
        let payload = service
            .generate(project_did, &json!(message), None, Utc::now())
            .await?;
        let payload = serde_json::to_string(&payload)?;
        let url = self.base_url.join(path)?;
        self.post(url.as_ref(), &payload).await
    }

    pub async fn get_message(
        &self,
        path: &str,
        project_did: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let payload = self
            .service
            .generate(project_did, &serde_json::Value::Null, None, Utc::now())
            .await?
            .to_string();
        let url = self.base_url.join(path)?;
        self.post(url.as_ref(), &payload).await
    }

    pub async fn ack_message(
        &self,
        path: &str,
        project_did: &str,
        message_id: String,
        is_verified: bool,
    ) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(path);
        let payload = json!({
            "message_id": message_id,
            "is_verified": is_verified,
        });
        let payload = self
            .service
            .generate(project_did, &payload, None, Utc::now())
            .await?
            .to_string();
        self.post(url.unwrap().as_ref(), &payload).await
    }

    pub async fn network(
        &self,
        path: &str,
        project_did: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let payload = self
            .service
            .generate(project_did, &serde_json::Value::Null, None, Utc::now())
            .await?;
        let payload = serde_json::to_string(&payload)?;
        self.post(path, &payload).await
    }

    pub async fn put(&self, path: &str, body: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        let headers = self.auth_headers("".to_string())?;
        let response = self
            .instance
            .put(url)
            .headers(headers)
            .body(body.to_string())
            .send()
            .await?;

        Ok(response)
    }

    #[allow(dead_code)]
    pub async fn delete(&self, _path: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(_path)?;
        let headers = self.auth_headers("".to_string())?;
        let response = self.instance.delete(url).headers(headers).send().await?;

        Ok(response)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Res {
        origin: String,
    }

    #[actix_rt::test]
    #[ignore]
    async fn it_should_success_get() {
        let client_config: StudioClientConfig = StudioClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match StudioClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let res = match client.get("/get").await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let json: Res = match res.json().await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(!json.origin.is_empty());
    }

    #[actix_rt::test]
    #[ignore]
    async fn it_should_success_post() {
        let client_config: StudioClientConfig = StudioClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match StudioClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let res = match client.post("/post", r#"{"key":"value"}"#).await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let json: Res = match res.json().await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(!json.origin.is_empty());
    }

    #[actix_rt::test]
    #[ignore]
    async fn it_should_success_put() {
        let client_config: StudioClientConfig = StudioClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match StudioClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let res = match client.put("/put", r#"{"key":"value"}"#).await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let json: Res = match res.json().await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(!json.origin.is_empty());
    }

    #[actix_rt::test]
    #[ignore]
    async fn it_should_success_delete() {
        let client_config: StudioClientConfig = StudioClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match StudioClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let res = match client.delete("/delete").await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let json: Res = match res.json().await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(!json.origin.is_empty());
    }
}
