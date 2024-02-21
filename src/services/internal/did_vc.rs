use crate::{
    nodex::{
        cipher::credential_signer::{
            CredentialSigner, CredentialSignerError, CredentialSignerSuite,
        },
        keyring::{self},
        schema::general::{CredentialSubject, GeneralVcDataModel, Issuer},
    },
    repository::did_repository::DidRepository,
};
use anyhow::Context;
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

pub struct DIDVCService {
    did_repository: Box<dyn DidRepository + Send + Sync + 'static>,
}

impl DIDVCService {
    pub fn new<R: DidRepository + Send + Sync + 'static>(did_repository: R) -> Self {
        Self {
            did_repository: Box::new(did_repository),
        }
    }
}

#[derive(Debug, Error)]
pub enum DIDVCServiceError {
    #[error("key pairing error")]
    KeyParingError(#[from] keyring::keypair::KeyPairingError),
    #[error("failed to get my did")]
    MyDIDNotSet,
    #[error("credential signer error")]
    CredentialSignerError(#[from] CredentialSignerError),
}

impl DIDVCService {
    pub fn generate(
        &self,
        message: &Value,
        issuance_date: DateTime<Utc>,
    ) -> Result<GeneralVcDataModel, DIDVCServiceError> {
        let keyring = keyring::keypair::KeyPairing::load_keyring()?;
        let did = keyring
            .get_identifier()
            .map_err(|_| DIDVCServiceError::MyDIDNotSet)?;

        let r#type = "VerifiableCredential".to_string();
        let context = "https://www.w3.org/2018/credentials/v1".to_string();
        let issuance_date = issuance_date.to_rfc3339();

        let model = GeneralVcDataModel {
            id: None,
            issuer: Issuer { id: did.clone() },
            r#type: vec![r#type],
            context: vec![context],
            issuance_date,
            credential_subject: CredentialSubject {
                id: None,
                container: message.clone(),
            },
            expiration_date: None,
            proof: None,
        };

        let signed: GeneralVcDataModel = CredentialSigner::sign(
            &model,
            &CredentialSignerSuite {
                did: Some(did),
                key_id: Some("signingKey".to_string()),
                context: keyring.get_sign_key_pair(),
            },
        )?;

        Ok(signed)
    }

    pub async fn verify(&self, model: GeneralVcDataModel) -> anyhow::Result<GeneralVcDataModel> {
        let did_document = self
            .did_repository
            .find_identifier(&model.issuer.id)
            .await?
            .with_context(|| format!("did {} not found", &model.issuer.id))?;
        let public_keys = did_document
            .did_document
            .public_key
            .ok_or(anyhow::anyhow!("public_key is not found in did_document"))?;

        // FIXME: workaround
        anyhow::ensure!(public_keys.len() == 1, "public_keys length must be 1");

        let public_key = public_keys[0].clone();

        let context = keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk)?;

        let (verified_model, verified) = CredentialSigner::verify(
            model,
            &CredentialSignerSuite {
                did: None,
                key_id: None,
                context,
            },
        )
        .context("failed to verify credential")?;

        anyhow::ensure!(verified, "signature is not verified");

        Ok(verified_model)
    }
}
