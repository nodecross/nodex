use crate::nodex::{
    errors::NodeXError,
    keyring::{self},
    runtime::base64_url::{self, PaddingType},
    schema::general::GeneralVcDataModel,
};
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
    ) -> Result<Value, NodeXError> {
        let keyring = match keyring::keypair::KeyPairing::load_keyring() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(NodeXError {});
            }
        };
        let did = match keyring.get_identifier() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(NodeXError {});
            }
        };

        let body = match DIDVCService::generate(message) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(NodeXError {});
            }
        };

        let mut message = match Message::new()
            .from(&did)
            .to(&[to_did])
            .body(&body.to_string())
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to initialize message with error = {:?}", e);
                return Err(NodeXError {});
            }
        };

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

        match message.clone().as_jws(&SignatureAlgorithm::Es256k).sign(
            SignatureAlgorithm::Es256k.signer(),
            &keyring.get_sign_key_pair().get_secret_key(),
        ) {
            Ok(v) => match serde_json::from_str::<Value>(&v) {
                Ok(v) => Ok(v),
                Err(e) => {
                    log::error!("{:?}", e);
                    Err(NodeXError {})
                }
            },
            Err(e) => {
                log::error!("{:?}", e);
                Err(NodeXError {})
            }
        }
    }

    pub async fn verify(message: &Value) -> Result<VerifiedContainer, NodeXError> {
        let service = crate::services::nodex::NodeX::new();

        let payload = match message.get("payload") {
            Some(v) => match v.as_str() {
                Some(v) => v.to_string(),
                None => return Err(NodeXError {}),
            },
            None => return Err(NodeXError {}),
        };

        let decoded =
            match base64_url::Base64Url::decode_as_string(&payload, &PaddingType::NoPadding) {
                Ok(v) => match serde_json::from_str::<Value>(&v) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("{:?}", e);
                        return Err(NodeXError {});
                    }
                },
                Err(e) => {
                    log::error!("{:?}", e);
                    return Err(NodeXError {});
                }
            };

        let from_did = match decoded.get("from") {
            Some(v) => match v.as_str() {
                Some(v) => v.to_string(),
                None => return Err(NodeXError {}),
            },
            None => return Err(NodeXError {}),
        };

        let did_document = match service.find_identifier(&from_did).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(NodeXError {});
            }
        };
        let public_keys = match did_document.did_document.public_key {
            Some(v) => v,
            None => return Err(NodeXError {}),
        };

        // FIXME: workaround
        if public_keys.len() != 1 {
            return Err(NodeXError {});
        }

        let public_key = public_keys[0].clone();

        let context = match keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(NodeXError {});
            }
        };

        let message =
            match Message::verify(message.to_string().as_bytes(), &context.get_public_key()) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("{:?}", e);
                    return Err(NodeXError {});
                }
            };

        let metadata = message
            .attachment_iter()
            .find(|item| match item.format.clone() {
                Some(value) => value == "metadata",
                None => false,
            });

        let body = match message.clone().get_body() {
            Ok(v) => match serde_json::from_str::<GeneralVcDataModel>(&v) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("{:?}", e);
                    return Err(NodeXError {});
                }
            },
            Err(e) => {
                log::error!("{:?}", e);
                return Err(NodeXError {});
            }
        };

        match metadata {
            Some(metadata) => match metadata.data.json.clone() {
                Some(json) => match serde_json::from_str::<Value>(&json) {
                    Ok(metadata) => Ok(VerifiedContainer {
                        message: body,
                        metadata: Some(metadata),
                    }),
                    _ => Err(NodeXError {}),
                },
                _ => Err(NodeXError {}),
            },
            None => Ok(VerifiedContainer {
                message: body,
                metadata: None,
            }),
        }
    }
}
