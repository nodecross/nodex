use crate::{
    nodex::{
        keyring::{self},
        runtime::base64_url::{self, PaddingType},
        schema::general::GeneralVcDataModel,
    },
    services::nodex::NodeX,
};
use anyhow::Context;
use chrono::{DateTime, Utc};
use cuid;
use didcomm_rs::{
    crypto::{SignatureAlgorithm, Signer},
    AttachmentBuilder, AttachmentDataBuilder, Message,
};
use serde_json::Value;

use super::{attachment_link, did_vc::DIDVCService, types::VerifiedContainer};

pub struct DIDCommSignedService {}

impl DIDCommSignedService {
    pub fn generate(
        to_did: &str,
        message: &Value,
        metadata: Option<&Value>,
        issuance_date: DateTime<Utc>,
    ) -> anyhow::Result<Value> {
        let keyring = keyring::keypair::KeyPairing::load_keyring()?;
        let did = keyring.get_identifier()?;

        let body = DIDVCService::new(NodeX::new()).generate(message, issuance_date)?;

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

        let jws = message
            .clone()
            .as_jws(&SignatureAlgorithm::Es256k)
            .sign(
                SignatureAlgorithm::Es256k.signer(),
                &keyring.get_sign_key_pair().get_secret_key(),
            )
            .map_err(|e| anyhow::anyhow!("failed to convert to jws. error = {:?}", e))?;

        Ok(serde_json::from_str::<Value>(&jws)?)
    }

    pub async fn verify(message: &Value) -> anyhow::Result<VerifiedContainer> {
        let service = crate::services::nodex::NodeX::new();

        let payload = message
            .get("payload")
            .ok_or(anyhow::anyhow!("No payload"))?
            .as_str()
            .ok_or(anyhow::anyhow!("failed to convert to str"))?;

        let decoded = base64_url::Base64Url::decode_as_string(payload, &PaddingType::NoPadding)?;
        let decoded = serde_json::from_str::<Value>(&decoded)?;

        let from_did = decoded
            .get("from")
            .ok_or(anyhow::anyhow!("No from"))?
            .as_str()
            .ok_or(anyhow::anyhow!("failed to convert to str"))?;

        let did_document = service
            .find_identifier(from_did)
            .await?
            .context(format!("did {} not found", from_did))?;

        let public_keys = did_document
            .did_document
            .public_key
            .ok_or(anyhow::anyhow!("public_key is not found in did_document"))?;

        // FIXME: workaround
        anyhow::ensure!(public_keys.len() == 1, "public_key length must be 1");

        let public_key = public_keys[0].clone();

        let context = keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk)?;

        let message = Message::verify(message.to_string().as_bytes(), &context.get_public_key())
            .map_err(|e| anyhow::anyhow!("failed to verify message. error = {:?}", e))?;

        let metadata = message
            .attachment_iter()
            .find(|item| match item.format.clone() {
                Some(value) => value == "metadata",
                None => false,
            });

        let body = message
            .get_body()
            .map_err(|e| anyhow::anyhow!("failed to get body. error = {:?}", e))?;
        let body = serde_json::from_str::<GeneralVcDataModel>(&body)?;

        match metadata {
            Some(metadata) => {
                let data = metadata
                    .data
                    .json
                    .clone()
                    .ok_or(anyhow::anyhow!("metadata not found"))?;
                let metadata = serde_json::from_str::<Value>(&data)?;
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
