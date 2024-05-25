use crate::repository::message_activity_repository::MessageActivityHttpError;
use crate::repository::metric_repository::{MetricStoreRepository, MetricStoreRequest};
use crate::server_config;
use crate::{
    nodex::utils::studio_client::{StudioClient, StudioClientConfig},
    repository::message_activity_repository::{
        CreatedMessageActivityRequest, MessageActivityRepository, VerifiedMessageActivityRequest,
    },
};

use anyhow::Context;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{
    internal::{did_vc::DIDVCService, didcomm_encrypted::DIDCommEncryptedService},
    nodex::NodeX,
};

#[derive(Deserialize)]
pub struct EmptyResponse {}

#[derive(Deserialize, Debug, Clone)]
pub struct MessageResponse {
    pub id: String,
    pub raw_message: String,
}

#[derive(Deserialize, Debug, Clone)]
struct ErrorResponse {
    pub message: String,
}

pub struct Studio {
    http_client: StudioClient,
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

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkResponse {
    pub secret_key: String,
    pub project_did: String,
    pub recipient_dids: Vec<String>,
    pub studio_endpoint: String,
    pub heartbeat: u64,
}

impl Studio {
    pub fn new() -> Self {
        let server_config = server_config();
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

        Studio {
            http_client: client,
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
        let res = self.http_client.post("/v1/device", &payload).await?;

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
        project_did: String,
        mac_address: String,
        version: String,
        os: String,
    ) -> anyhow::Result<()> {
        let res = self
            .http_client
            .send_device_info("/v1/device-info", &project_did, &mac_address, &version, &os)
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

    pub async fn get_message(&self, project_did: &str) -> anyhow::Result<Vec<MessageResponse>> {
        let res = self
            .http_client
            .get_message("/v1/message/list", project_did)
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => match res.json::<Vec<MessageResponse>>().await {
                Ok(v) => Ok(v),
                Err(e) => anyhow::bail!("StatusCode=200, but parse failed. {:?}", e),
            },
            reqwest::StatusCode::BAD_REQUEST => match res.json::<ErrorResponse>().await {
                Ok(v) => anyhow::bail!("StatusCode=400, error message = {:?}", v.message),
                Err(e) => anyhow::bail!("StatusCode=400, but parse failed. {:?}", e),
            },
            other => anyhow::bail!("StatusCode={other}, unexpected response"),
        }
    }

    pub async fn ack_message(
        &self,
        project_did: &str,
        message_id: String,
        is_verified: bool,
    ) -> anyhow::Result<()> {
        let res = self
            .http_client
            .ack_message("/v1/message/ack", project_did, message_id, is_verified)
            .await?;

        res.json::<EmptyResponse>().await?;
        Ok(())
    }

    pub async fn network(&self) -> anyhow::Result<()> {
        let project_did = {
            let network = crate::network_config();
            let network = network.lock();
            network.get_project_did().expect("project_did is not set")
        };

        let res = self
            .http_client
            .network("/v1/network", &project_did)
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => match res.json::<NetworkResponse>().await {
                Ok(v) => {
                    let network = crate::network_config();
                    let mut network = network.lock();
                    network.save_secret_key(&v.secret_key);
                    network.save_project_did(&v.project_did);
                    network.save_recipient_dids(v.recipient_dids);
                    network.save_studio_endpoint(&v.studio_endpoint);
                    network.save_heartbeat(v.heartbeat);
                    Ok(())
                }
                Err(e) => anyhow::bail!("StatusCode=200, but parse failed. {:?}", e),
            },
            reqwest::StatusCode::BAD_REQUEST => match res.json::<ErrorResponse>().await {
                Ok(v) => anyhow::bail!("StatusCode=400, error message = {:?}", v.message),
                Err(e) => anyhow::bail!("StatusCode=400, but parse failed. {:?}", e),
            },
            other => anyhow::bail!("StatusCode={other}, unexpected response"),
        }
    }
}

#[async_trait::async_trait]
impl MessageActivityRepository for Studio {
    async fn add_create_activity(
        &self,
        request: CreatedMessageActivityRequest,
    ) -> Result<(), MessageActivityHttpError> {
        // TODO: refactoring more simple
        let service = DIDCommEncryptedService::new(NodeX::new(), DIDVCService::new(NodeX::new()));

        let project_did = {
            let network = crate::network_config();
            let network = network.lock();
            network.get_project_did().expect("project_did is not set")
        };
        let payload = service
            .generate(&project_did, &json!(request), None, request.occurred_at)
            .await
            .context("failed to generate payload")?;
        let payload = serde_json::to_string(&payload).context("failed to serialize")?;

        let res = self
            .http_client
            .post("/v1/message-activity", &payload)
            .await?;

        let status = res.status();
        let json: Value = res.json().await.context("Failed to read response body")?;
        let message = if let Some(message) = json.get("message").map(|v| v.to_string()) {
            message
        } else {
            "".to_string()
        };

        match status {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::BAD_REQUEST => Err(MessageActivityHttpError::BadRequest(message)),
            reqwest::StatusCode::UNAUTHORIZED => {
                Err(MessageActivityHttpError::Unauthorized(message))
            }
            reqwest::StatusCode::FORBIDDEN => Err(MessageActivityHttpError::Forbidden(message)),
            reqwest::StatusCode::NOT_FOUND => Err(MessageActivityHttpError::NotFound(message)),
            reqwest::StatusCode::CONFLICT => Err(MessageActivityHttpError::Conflict(message)),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                Err(MessageActivityHttpError::InternalServerError(message))
            }
            other => Err(MessageActivityHttpError::Other(anyhow::anyhow!(
                "StatusCode={}, unexpected response",
                other
            ))),
        }
    }

    async fn add_verify_activity(
        &self,
        request: VerifiedMessageActivityRequest,
    ) -> Result<(), MessageActivityHttpError> {
        // TODO: refactoring more simple
        let service = DIDCommEncryptedService::new(NodeX::new(), DIDVCService::new(NodeX::new()));
        let project_did = {
            let network = crate::network_config();
            let network = network.lock();
            network.get_project_did().expect("project_did is not set")
        };
        let payload = service
            .generate(&project_did, &json!(request), None, request.verified_at)
            .await
            .context("failed to generate payload")?;
        let payload = serde_json::to_string(&payload).context("failed to serialize")?;

        let res = self
            .http_client
            .put("/v1/message-activity", &payload)
            .await?;

        let status = res.status();
        let json: Value = res.json().await.context("Failed to read response body")?;
        let message = if let Some(message) = json.get("message").map(|v| v.to_string()) {
            message
        } else {
            "".to_string()
        };

        match status {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::BAD_REQUEST => Err(MessageActivityHttpError::BadRequest(message)),
            reqwest::StatusCode::UNAUTHORIZED => {
                Err(MessageActivityHttpError::Unauthorized(message))
            }
            reqwest::StatusCode::FORBIDDEN => Err(MessageActivityHttpError::Forbidden(message)),
            reqwest::StatusCode::NOT_FOUND => Err(MessageActivityHttpError::NotFound(message)),
            reqwest::StatusCode::CONFLICT => Err(MessageActivityHttpError::Conflict(message)),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                Err(MessageActivityHttpError::InternalServerError(message))
            }
            other => Err(MessageActivityHttpError::Other(anyhow::anyhow!(
                "StatusCode={}, unexpected response",
                other
            ))),
        }
    }
}

#[async_trait::async_trait]
impl MetricStoreRepository for Studio {
    async fn save(&self, request: MetricStoreRequest) -> anyhow::Result<()> {
        let payload = serde_json::to_string(&request).expect("failed to serialize");
        let res = self.http_client.post("/v1/metric", &payload).await?;

        let status = res.status();
        let json: Value = res.json().await.context("Failed to read response body")?;
        let message = if let Some(message) = json.get("message").map(|v| v.to_string()) {
            message
        } else {
            "".to_string()
        };

        match status {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::NOT_FOUND => anyhow::bail!("StatusCode=404, {}", message),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                anyhow::bail!("StatusCode=500, {}", message);
            }
            other => anyhow::bail!("StatusCode={other}, {}", message),
        }
    }
}
