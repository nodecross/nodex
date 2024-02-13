use crate::server_config;
use crate::{
    nodex::{
        keyring,
        sidetree::payload::{
            CommitmentKeys, DIDCreateRequest, DIDResolutionResponse, OperationPayloadBuilder,
        },
        utils::http_client::{HttpClient, HttpClientConfig},
    },
    repository::did_repository::DidRepository,
};

use reqwest::StatusCode;
use serde::Deserialize;
use std::{fs, process::Command};
use thiserror::Error;

pub struct NodeX {
    http_client: HttpClient,
}

#[derive(Debug, Deserialize)]
pub struct SideTreeErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Error)]
#[error("SideTreeError: {status_code}")]
pub struct SideTreeError {
    pub status_code: StatusCode,
    pub error: SideTreeErrorBody,
}

impl NodeX {
    pub fn new() -> Self {
        let server_config = server_config();
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: server_config.did_http_endpoint(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                panic!()
            }
        };

        NodeX {
            http_client: client,
        }
    }

    // NOTE: DONE
    pub async fn create_identifier(&self) -> anyhow::Result<DIDResolutionResponse> {
        // NOTE: find did
        if let Ok(v) = keyring::keypair::KeyPairing::load_keyring() {
            if let Ok(did) = v.get_identifier() {
                if let Some(json) = self.find_identifier(&did).await? {
                    return Ok(json);
                }
            }
        }

        // NOTE: does not exists did key ring
        let mut keyring = keyring::keypair::KeyPairing::create_keyring()?;

        // NOTE: create payload
        let public = keyring
            .get_sign_key_pair()
            .to_public_key("signingKey", &["auth", "general"])?;
        let update = keyring.get_recovery_key_pair().to_jwk(false)?;
        let recovery = keyring.get_update_key_pair().to_jwk(false)?;
        let payload = OperationPayloadBuilder::did_create_payload(&DIDCreateRequest {
            public_keys: vec![public],
            commitment_keys: CommitmentKeys { recovery, update },
            service_endpoints: vec![],
        })?;

        let res = self
            .http_client
            .post("/api/v1/operations", &payload)
            .await?;

        if res.status().is_success() {
            let json = res.json::<DIDResolutionResponse>().await?;

            // NOTE: save context
            keyring.save(&json.did_document.id);

            Ok(json)
        } else {
            let status = res.status();
            let error = res.json::<SideTreeErrorBody>().await?;
            Err(SideTreeError {
                status_code: status,
                error,
            }
            .into())
        }
    }

    // NOTE: DONE
    pub async fn find_identifier(
        &self,
        did: &str,
    ) -> anyhow::Result<Option<DIDResolutionResponse>> {
        let res = self
            .http_client
            .get(&(format!("/api/v1/identifiers/{}", &did)))
            .await?;

        match res.status() {
            StatusCode::OK => Ok(Some(res.json::<DIDResolutionResponse>().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            other => {
                let error = res.json::<SideTreeErrorBody>().await?;
                Err(SideTreeError {
                    status_code: other,
                    error,
                }
                .into())
            }
        }
    }

    pub async fn update_version(&self, binary_url: &str, path: &str) -> anyhow::Result<()> {
        let response = reqwest::get(binary_url).await?;
        let content = response.bytes().await?;

        fs::write(path, &content)?;

        Command::new("chmod").arg("+x").arg(path).status()?;
        Command::new(path).spawn()?;
        Ok(())
    }
}

// TODO: use other impl
#[async_trait::async_trait]
impl DidRepository for NodeX {
    async fn create_identifier(&self) -> anyhow::Result<DIDResolutionResponse> {
        NodeX::create_identifier(self).await
    }

    async fn find_identifier(&self, did: &str) -> anyhow::Result<Option<DIDResolutionResponse>> {
        NodeX::find_identifier(self, did).await
    }
}
