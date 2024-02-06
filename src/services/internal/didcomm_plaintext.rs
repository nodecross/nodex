use super::{attachment_link, did_vc::DIDVCService, types::VerifiedContainer};
use crate::nodex::{
    keyring::{self},
    schema::general::GeneralVcDataModel,
};
use chrono::{DateTime, Utc};
use cuid;
use didcomm_rs::{AttachmentBuilder, AttachmentDataBuilder, Message};
use serde_json::Value;

pub struct DIDCommPlaintextService {}

impl DIDCommPlaintextService {
    pub fn generate(
        to_did: &str,
        message: &Value,
        metadata: Option<&Value>,
        issuance_date: DateTime<Utc>,
    ) -> anyhow::Result<Value> {
        let keyring = keyring::keypair::KeyPairing::load_keyring()?;
        let did = keyring.get_identifier()?;

        let body = DIDVCService::generate(message, issuance_date)?;

        let mut message = Message::new()
            .from(&did)
            .to(&[to_did])
            .body(&body.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to initialize message with error = {:?}", e))?;

        // NOTE: Has attachment
        if let Some(value) = metadata {
            let id = cuid::cuid2();

            let data = AttachmentDataBuilder::new()
                .with_link(&attachment_link())
                .with_json(&value.to_string());

            message.append_attachment(
                AttachmentBuilder::new(true)
                    .with_id(&id)
                    .with_format("metadata")
                    .with_data(data),
            )
        }

        let raw_json = message
            .clone()
            .as_raw_json()
            .map_err(|e| anyhow::anyhow!("failed to serialize message. error = {:?}", e))?;
        Ok(serde_json::from_str(&raw_json)?)
    }

    pub fn verify(message: &Value) -> anyhow::Result<VerifiedContainer> {
        let message = Message::receive(&message.to_string(), None, None, None)
            .map_err(|e| anyhow::anyhow!("message construct failed. error = {:?}", e))?;

        let metadata = message
            .attachment_iter()
            .find(|item| match item.format.clone() {
                Some(value) => value == "metadata",
                None => false,
            });

        let body = message
            .clone()
            .get_body()
            .map_err(|e| anyhow::anyhow!("failed to get body. error = {:?}", e))?;
        let body = serde_json::from_str::<GeneralVcDataModel>(&body)?;

        match metadata {
            Some(metadata) => {
                let metadata = metadata
                    .data
                    .json
                    .clone()
                    .ok_or(anyhow::anyhow!("metadata not found"))?;
                let metadata = serde_json::from_str::<Value>(&metadata)?;
                Ok(VerifiedContainer {
                    message: body,
                    metadata: Some(metadata),
                })
            }
            None => Ok(VerifiedContainer {
                message: body,
                metadata: None,
            }),
        }
    }
}
