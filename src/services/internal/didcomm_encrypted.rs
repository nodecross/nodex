use super::{attachment_link, did_vc::DIDVCService, types::VerifiedContainer};
use crate::nodex::{
    keyring,
    runtime::{
        self,
        base64_url::{self, PaddingType},
    },
    schema::general::GeneralVcDataModel,
};
use arrayref::array_ref;
use chrono::{DateTime, Utc};
use cuid;
use didcomm_rs::{
    crypto::{CryptoAlgorithm, SignatureAlgorithm},
    AttachmentBuilder, AttachmentDataBuilder, Message,
};
use serde_json::Value;
use x25519_dalek::{PublicKey, StaticSecret};

pub struct DIDCommEncryptedService {}

impl DIDCommEncryptedService {
    pub async fn generate(
        to_did: &str,
        message: &Value,
        metadata: Option<&Value>,
        issuance_date: DateTime<Utc>,
    ) -> anyhow::Result<Value> {
        let service = crate::services::nodex::NodeX::new();

        // NOTE: recipient from
        let my_keyring = keyring::keypair::KeyPairing::load_keyring()?;
        let my_did = my_keyring.get_identifier()?;

        // NOTE: recipient to
        let did_document = service.find_identifier(to_did).await?;
        let public_keys = did_document
            .did_document
            .public_key
            .ok_or(anyhow::anyhow!("DidPublicKey not found"))?;

        // FIXME: workaround
        anyhow::ensure!(public_keys.len() == 1, "public_keys length must be 1");

        let public_key = public_keys[0].clone();

        let other_key = keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk)?;

        // NOTE: ecdh
        let shared_key = runtime::secp256k1::Secp256k1::ecdh(
            &my_keyring.get_sign_key_pair().get_secret_key(),
            &other_key.get_public_key(),
        )?;

        let sk = StaticSecret::from(array_ref!(shared_key, 0, 32).to_owned());
        let pk = PublicKey::from(&sk);

        // NOTE: message
        let body = DIDVCService::generate(message, issuance_date)?;

        let mut message = Message::new()
            .from(&my_did)
            .to(&[to_did])
            .body(&body.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to initialize message with error = {:?}", e))?;

        // NOTE: Has attachment
        if let Some(value) = metadata {
            let id = cuid::cuid2();

            // let media_type = "application/json";
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

        let seal_signed_message = message
            .clone()
            .as_jwe(&CryptoAlgorithm::XC20P, Some(pk.as_bytes().to_vec()))
            .seal_signed(
                sk.to_bytes().as_ref(),
                Some(vec![Some(pk.as_bytes().to_vec())]),
                SignatureAlgorithm::Es256k,
                &my_keyring.get_sign_key_pair().get_secret_key(),
            )
            .map_err(|e| anyhow::anyhow!("failed to encrypt message : {:?}", e))?;

        Ok(serde_json::from_str::<Value>(&seal_signed_message)?)
    }

    pub async fn verify(message: &Value) -> anyhow::Result<VerifiedContainer> {
        let service = crate::services::nodex::NodeX::new();

        // NOTE: recipient to
        let my_keyring = keyring::keypair::KeyPairing::load_keyring()?;

        // NOTE: recipient from
        let protected = message
            .get("protected")
            .ok_or(anyhow::anyhow!("protected not found"))?
            .as_str()
            .ok_or(anyhow::anyhow!("failed to serialize protected"))?;

        let decoded = base64_url::Base64Url::decode_as_string(protected, &PaddingType::NoPadding)?;
        let decoded = serde_json::from_str::<Value>(&decoded)?;

        let other_did = decoded
            .get("skid")
            .ok_or(anyhow::anyhow!("skid not found"))?
            .as_str()
            .ok_or(anyhow::anyhow!("failed to serialize skid"))?;

        let did_document = service.find_identifier(other_did).await?;

        let public_keys = did_document
            .did_document
            .public_key
            .ok_or(anyhow::anyhow!("public_key is not found in did_document"))?;

        // FIXME: workaround
        anyhow::ensure!(public_keys.len() == 1, "public_keys length must be 1");

        let public_key = public_keys[0].clone();

        let other_key = keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk)?;

        // NOTE: ecdh
        let shared_key = runtime::secp256k1::Secp256k1::ecdh(
            &my_keyring.get_sign_key_pair().get_secret_key(),
            &other_key.get_public_key(),
        )?;

        let sk = StaticSecret::from(array_ref!(shared_key, 0, 32).to_owned());
        let pk = PublicKey::from(&sk);

        let message = Message::receive(
            &message.to_string(),
            Some(sk.to_bytes().as_ref()),
            Some(pk.as_bytes().to_vec()),
            Some(&other_key.get_public_key()),
        )
        .map_err(|e| anyhow::anyhow!("failed to decrypt message : {:?}", e))?;

        let metadata = message
            .attachment_iter()
            .find(|item| match item.format.clone() {
                Some(value) => value == "metadata",
                None => false,
            });

        let body = message
            .clone()
            .get_body()
            .map_err(|e| anyhow::anyhow!("failed to get body : {:?}", e))?;
        let body = serde_json::from_str::<GeneralVcDataModel>(&body)?;

        match metadata {
            Some(metadata) => {
                let metadata = metadata
                    .data
                    .json
                    .as_ref()
                    .ok_or(anyhow::anyhow!("metadata not found"))?;
                let metadata = serde_json::from_str::<Value>(metadata)?;
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
