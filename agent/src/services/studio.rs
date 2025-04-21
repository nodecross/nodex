use crate::nodex::utils::did_accessor::{DidAccessor, DidAccessorImpl};
use crate::nodex::utils::studio_client::{StudioClient, StudioClientConfig};
use crate::repository::attribute_repository::{AttributeStoreRepository, AttributeStoreRequest};
use crate::repository::custom_metric_repository::CustomMetricStoreRepository;
use crate::repository::event_repository::EventStoreRepository;
use crate::repository::log_repository::LogStoreRepository;
use crate::repository::metric_repository::{MetricStoreRepository, MetricsWithTimestamp};
use crate::server_config;
use anyhow::Context;
use protocol::cbor::types::{CustomMetric, Event, Log};
use protocol::keyring::keypair::KeyPair;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;

// The maximum JSON body size is actually 1MB
// We reserve 100KB as a buffer for Verifiable Credential capacity
const JSON_BODY_MAX_SIZE: usize = 900_000;

#[derive(Deserialize)]
pub struct EmptyResponse {}

pub struct Studio {
    http_client: StudioClient,
    did_accessor: DidAccessorImpl,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterDeviceRequest {
    device_did: String,
    project_did: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendDeviceInfoRequest {
    device_did: String,
    mac_address: String,
    version: String,
    os: String,
}

impl Studio {
    pub fn new() -> Self {
        let server_config = server_config().expect("Failed to get server config");
        let client_config: StudioClientConfig = StudioClientConfig {
            base_url: server_config.studio_http_endpoint(),
        };

        let client = match StudioClient::new(&client_config) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                panic!()
            }
        };

        let did_accessor = DidAccessorImpl {};

        Studio {
            http_client: client,
            did_accessor,
        }
    }

    pub async fn register_device(
        &self,
        device_did: String,
        project_did: String,
    ) -> anyhow::Result<()> {
        let request = RegisterDeviceRequest {
            device_did,
            project_did,
        };
        let payload = serde_json::to_string(&request).expect("failed to serialize");
        let res = self
            .http_client
            .post_with_auth_header("/v1/device", payload)
            .await?;

        let status = res.status();

        let json: Value = res.json().await.context("Failed to read response body")?;
        let message = json
            .get("message")
            .map(|v| v.to_string())
            .unwrap_or_default();

        match status {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::BAD_REQUEST => {
                anyhow::bail!("StatusCode=400, {}", message)
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                anyhow::bail!("StatusCode=401, {}", message)
            }
            reqwest::StatusCode::NOT_FOUND => {
                anyhow::bail!("StatusCode=404, {}", message)
            }
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                anyhow::bail!("StatusCode=500, {}", message);
            }
            other => {
                anyhow::bail!("StatusCode={other}, {}", message);
            }
        }
    }

    pub async fn send_device_info(
        &self,
        mac_address: String,
        version: String,
        os: String,
    ) -> anyhow::Result<()> {
        let res = self
            .http_client
            .send_device_info("/v1/device-info", &mac_address, &version, &os)
            .await?;

        let status = res.status();

        let json: Value = res.json().await.context("Failed to read response body")?;
        let message = json
            .get("message")
            .map(|v| v.to_string())
            .unwrap_or_default();

        match status {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::BAD_REQUEST => {
                anyhow::bail!("StatusCode=400, {}", message)
            }
            reqwest::StatusCode::NOT_FOUND => {
                anyhow::bail!("StatusCode=404, {}", message)
            }
            other => {
                anyhow::bail!("StatusCode={other}, {}", message);
            }
        }
    }

    #[inline]
    async fn relay_to_studio_via_cbor<T: serde::Serialize + std::fmt::Debug>(
        &self,
        path: &str,
        request: T,
    ) -> anyhow::Result<()> {
        let my_did = self.did_accessor.get_my_did();
        let my_keyring = self.did_accessor.get_my_keyring();
        let signing_key = &my_keyring.sign_time_series.get_secret_key();
        let token = protocol::cbor::sign::Token::new(my_did);
        let request = protocol::cbor::sign::WithToken {
            inner: request,
            token,
        };
        let payload = protocol::cbor::sign::sign_message(signing_key, &request)?;
        let res = self.http_client.post_binary(path, payload).await?;
        let status = res.status();
        let json: Value = res.json().await.context("Failed to read response body")?;
        let message = json
            .get("message")
            .map(|v| v.to_string())
            .unwrap_or_default();
        match status {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::NOT_FOUND => anyhow::bail!("StatusCode=404, {}", message),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                anyhow::bail!("StatusCode=500, {}", message)
            }
            other => anyhow::bail!("StatusCode={other}, {}", message),
        }
    }

    // #[inline]
    // async fn relay_to_studio<T: serde::Serialize>(
    //     &self,
    //     path: &str,
    //     request: T,
    // ) -> anyhow::Result<()> {
    //     let my_did = self.did_accessor.get_my_did();
    //     let my_keyring = self.did_accessor.get_my_keyring();
    //     let model = VerifiableCredentials::new(
    //         my_did.into_inner(),
    //         serde_json::to_value(request)?,
    //         chrono::Utc::now(),
    //     );
    //     let payload = DidVcService::generate(&self.did_repository, model, &my_keyring)
    //         .context("failed to generate payload")?;
    //     let payload = serde_json::to_string(&payload).context("failed to serialize")?;
    //     let res = self.http_client.post(path, payload).await?;
    //     let status = res.status();
    //     let json: Value = res.json().await.context("Failed to read response body")?;
    //     let message = json
    //         .get("message")
    //         .map(|v| v.to_string())
    //         .unwrap_or_default();
    //     match status {
    //         reqwest::StatusCode::OK => Ok(()),
    //         reqwest::StatusCode::NOT_FOUND => anyhow::bail!("StatusCode=404, {}", message),
    //         reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
    //             anyhow::bail!("StatusCode=500, {}", message)
    //         }
    //         other => anyhow::bail!("StatusCode={other}, {}", message),
    //     }
    // }
}

impl MetricStoreRepository for Studio {
    async fn save(&self, request: VecDeque<MetricsWithTimestamp>) -> anyhow::Result<()> {
        let mut metrics = request;
        while !metrics.is_empty() {
            let my_did = self.did_accessor.get_my_did();
            let my_keyring = self.did_accessor.get_my_keyring();
            let signing_key = &my_keyring.sign_time_series.get_secret_key();

            let mut metrics_dst = Vec::new();
            let mut current_size = 0;

            while let Some(m) = metrics.pop_front() {
                let item_size = protocol::cbor::sign::to_message(&m)?.len();
                if item_size > JSON_BODY_MAX_SIZE {
                    anyhow::bail!("invalid item size: JSON body size too large")
                }
                if current_size + item_size > JSON_BODY_MAX_SIZE {
                    metrics.push_front(m);
                    break;
                }
                current_size += item_size;
                metrics_dst.push(m);
            }

            let token = protocol::cbor::sign::Token::new(my_did);
            let payload = protocol::cbor::sign::WithToken {
                inner: metrics_dst,
                token,
            };
            let payload = protocol::cbor::sign::sign_message(signing_key, &payload)?;
            let res = self.http_client.post_binary("/v1/metrics", payload).await?;

            let status = res.status();
            let json: Value = res.json().await.context("Failed to read response body")?;
            let message = if let Some(message) = json.get("message").map(|v| v.to_string()) {
                message
            } else {
                "".to_string()
            };
            match status {
                reqwest::StatusCode::OK => continue,
                reqwest::StatusCode::NOT_FOUND => anyhow::bail!("StatusCode=404, {}", message),
                reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                    anyhow::bail!("StatusCode=500, {}", message);
                }
                other => anyhow::bail!("StatusCode={other}, {}", message),
            }
        }

        Ok(())
    }
}

impl LogStoreRepository for Studio {
    async fn save(&self, request: Log) -> anyhow::Result<()> {
        self.relay_to_studio_via_cbor("/v1/logs", request).await
    }
}

impl EventStoreRepository for Studio {
    async fn save(&self, request: Vec<Event>) -> anyhow::Result<()> {
        self.relay_to_studio_via_cbor("/v1/events", request).await
    }
}

impl CustomMetricStoreRepository for Studio {
    async fn save(&self, request: Vec<CustomMetric>) -> anyhow::Result<()> {
        self.relay_to_studio_via_cbor("/v1/custom-metrics", request)
            .await
    }
}

impl AttributeStoreRepository for Studio {
    async fn save(&self, request: AttributeStoreRequest) -> anyhow::Result<()> {
        self.relay_to_studio_via_cbor("/v1/tag-values", request)
            .await
    }
}
