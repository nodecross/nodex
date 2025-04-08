use super::did_accessor::{DidAccessor, DidAccessorImpl};
use crate::network_config;
use backon::ExponentialBuilder;
use backon::Retryable;
use hmac::{Hmac, Mac};
use protocol::keyring::keypair::KeyPair;
use reqwest::Body;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Url,
};
use serde_json::json;
use sha2::Sha256;

#[derive(Debug, thiserror::Error)]
enum SendToStudioError {
    #[error("Failed to clone request")]
    Clone,
    #[error(transparent)]
    Others(#[from] reqwest::Error),
    #[error("Retry to send request")]
    Retry,
}

type HmacSha256 = Hmac<Sha256>;

pub struct StudioClientConfig {
    pub base_url: Url,
}

pub struct StudioClient {
    pub base_url: Url,
    pub instance: reqwest::Client,
    pub did_accessor: DidAccessorImpl,
}

impl StudioClient {
    pub fn new(_config: &StudioClientConfig) -> anyhow::Result<Self> {
        let url = _config.base_url.clone();
        let client = reqwest::Client::new();
        let did_accessor = DidAccessorImpl {};

        Ok(StudioClient {
            instance: client,
            base_url: url,
            did_accessor,
        })
    }

    fn auth_headers(&self, payload: &[u8]) -> anyhow::Result<HeaderMap> {
        let config = network_config();
        let secret = config
            .lock()
            .get_secret_key()
            .ok_or(anyhow::anyhow!("not found secret key"))?;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
        mac.update(payload);
        let signature = &hex::encode(mac.finalize().into_bytes());
        let mut headers = HeaderMap::new();
        headers.insert("X-Nodex-Signature", HeaderValue::from_str(signature)?);
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        Ok(headers)
    }

    async fn _execute(
        &self,
        request: &reqwest::Request,
    ) -> Result<reqwest::Response, SendToStudioError> {
        let response = self
            .instance
            .execute(request.try_clone().ok_or(SendToStudioError::Clone)?)
            .await?;
        match response.status() {
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => Err(SendToStudioError::Retry),
            _ => Ok(response),
        }
    }

    async fn _post_common(
        &self,
        path: &str,
        body: Body,
        headers: HeaderMap,
    ) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        let request = self
            .instance
            .post(url)
            .headers(headers)
            .body(body)
            .build()?;
        let execute = || self._execute(&request);
        let response = execute
            .retry(ExponentialBuilder::default())
            .sleep(tokio::time::sleep)
            .when(|e| matches!(e, SendToStudioError::Retry))
            .await?;
        Ok(response)
    }

    async fn _post_with_auth_header(
        &self,
        path: &str,
        body: Body,
    ) -> anyhow::Result<reqwest::Response> {
        let headers = self.auth_headers(body.as_bytes().ok_or(anyhow::anyhow!("invalid body"))?)?;
        self._post_common(path, body, headers).await
    }

    pub async fn post_with_auth_header(
        &self,
        path: &str,
        body: impl Into<Body>,
    ) -> anyhow::Result<reqwest::Response> {
        self._post_with_auth_header(path, body.into()).await
    }

    async fn _post(&self, path: &str, body: Body) -> anyhow::Result<reqwest::Response> {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        self._post_common(path, body, headers).await
    }

    // pub async fn post(
    //     &self,
    //     path: &str,
    //     body: impl Into<Body>,
    // ) -> anyhow::Result<reqwest::Response> {
    //     self._post(path, body.into()).await
    // }

    async fn _post_binary(&self, path: &str, body: Body) -> anyhow::Result<reqwest::Response> {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/cose; cose-type=\"cose-sign1\""),
        );
        self._post_common(path, body, headers).await
    }

    pub async fn post_binary(
        &self,
        path: &str,
        body: impl Into<Body>,
    ) -> anyhow::Result<reqwest::Response> {
        self._post_binary(path, body.into()).await
    }

    pub async fn send_device_info(
        &self,
        path: &str,
        mac_address: &str,
        version: &str,
        os: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let message = json!({
            "mac_address": mac_address,
            "version": version,
            "os": os,
        });
        let my_did = self.did_accessor.get_my_did();
        let token = protocol::cbor::sign::Token::new(my_did);
        let my_keyring = self.did_accessor.get_my_keyring();
        let key = my_keyring.sign_time_series.get_secret_key();
        let request = protocol::cbor::sign::WithToken {
            inner: message,
            token,
        };
        let payload = protocol::cbor::sign::sign_message(&key, &request)?;
        let url = self.base_url.join(path)?;
        self.post_binary(url.as_str(), payload).await
    }

    // pub async fn put(&self, path: &str, body: &str) -> anyhow::Result<reqwest::Response> {
    //     let url = self.base_url.join(path)?;
    //     let mut headers = HeaderMap::new();
    //     headers.insert(
    //         reqwest::header::CONTENT_TYPE,
    //         HeaderValue::from_static("application/json"),
    //     );

    //     let response = self
    //         .instance
    //         .put(url)
    //         .headers(headers)
    //         .body(body.to_string())
    //         .send()
    //         .await?;

    //     Ok(response)
    // }
}

// #[cfg(test)]
// pub mod tests {
//     use super::*;
//     use serde::Deserialize;

//     #[derive(Deserialize)]
//     struct Res {
//         origin: String,
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn it_should_success_post() {
//         let client_config: StudioClientConfig = StudioClientConfig {
//             base_url: Url::parse("https://httpbin.org").unwrap(),
//         };

//         let client = match StudioClient::new(&client_config) {
//             Ok(v) => v,
//             Err(_) => panic!(),
//         };

//         let res = match client.post("/post", r#"{"key":"value"}"#).await {
//             Ok(v) => v,
//             Err(_) => panic!(),
//         };

//         let json: Res = match res.json().await {
//             Ok(v) => v,
//             Err(_) => panic!(),
//         };

//         assert!(!json.origin.is_empty());
//     }
// }
