use serde_json::Value;
use didcomm_rs::{Message, crypto::{SignatureAlgorithm, Signer}, AttachmentBuilder, AttachmentDataBuilder};
use cuid;
use crate::{unid::{errors::UNiDError, keyring::{self}, runtime::base64_url::{self, PaddingType}}};

use super::{did_vc::DIDVCService, types::VerifiedContainer};

pub struct DIDCommSignedService {}

impl DIDCommSignedService {
    pub fn generate(to_did: &str, message: &Value, metadata: Option<&Value>) -> Result<Value, UNiDError> {
        let keyring = match keyring::mnemonic::MnemonicKeyring::load_keyring() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let did = match keyring.get_identifier() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let body = match DIDVCService::generate(&message) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let mut message = Message::new()
            .from(&did)
            .to(&[ &to_did ])
            .body(&body.to_string());

        // NOTE: Has attachment
        if let Some(value) = metadata {
            let id = match cuid::cuid() {
                Ok(v) => v,
                _ => return Err(UNiDError{}),
            };

            let data = AttachmentDataBuilder::new()
                .with_link("https://did.getunid.io")
                .with_json(&value.to_string());

            message.apeend_attachment(
                AttachmentBuilder::new(true)
                .with_id(&id)
                .with_format("metadata")
                .with_data(data)
            )
        }

        match message.clone()
            .as_jws(&SignatureAlgorithm::Es256k)
            .sign(SignatureAlgorithm::Es256k.signer(), &keyring.get_sign_key_pair().get_secret_key()) {
                Ok(v) => {
                    match serde_json::from_str::<Value>(&v) {
                        Ok(v) => Ok(v),
                        Err(_) => Err(UNiDError{}),
                    }
                },
                Err(_) => Err(UNiDError{})
            }
    }

    pub async fn verify(message: &Value) -> Result<VerifiedContainer, UNiDError> {
        let service = crate::services::unid::UNiD::new();

        let payload = match message.get("payload") {
            Some(v) => {
                match v.as_str() {
                    Some(v) => v.to_string(),
                    None => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };

        let decoded = match base64_url::Base64Url::decode_as_string(&payload, &PaddingType::NoPadding) {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        };

        let from_did = match decoded.get("from") {
            Some(v) => {
                match v.as_str() {
                    Some(v) => v.to_string(),
                    None => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };

        let did_document = match service.find_identifier(&from_did).await {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };
        let public_keys = match did_document.did_document.public_key {
            Some(v) => v,
            None => return Err(UNiDError{}),
        };

        // FIXME: workaround
        if public_keys.len() != 1 {
            return Err(UNiDError{})
        }

        let public_key = public_keys[0].clone();

        let context = match keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let message = match Message::verify(&message.to_string().as_bytes(), &context.get_public_key()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let metadata = message
            .get_attachments()
            .find(|item| {
                match item.format.clone() {
                    Some(value) => value == "metadata",
                    None => false
                }
            });

        let body = match message.clone().get_body() {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        };

        match metadata {
            Some(metadata) => {
                match metadata.data.json.clone() {
                    Some(json) => {
                        match serde_json::from_str::<Value>(&json) {
                            Ok(metadata) => {
                                Ok(VerifiedContainer {
                                    message: body,
                                    metadata: Some(metadata),
                                })
                            },
                            _ => Err(UNiDError {})
                        }
                    },
                    _ => Err(UNiDError {})
                }
            },
            None => {
                Ok(VerifiedContainer {
                    message: body,
                    metadata: None,
                })
            }
        }
    }
}