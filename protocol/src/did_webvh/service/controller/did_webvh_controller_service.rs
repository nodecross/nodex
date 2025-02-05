use crate::did_webvh::domain::crypto::crypto_utils::multibase_encode;
use crate::did_webvh::domain::did_document::{DidDocument, VerificationMethod};
use crate::did_webvh::domain::did_log_entry::DidLogEntry;
use crate::did_webvh::infra::did_webvh_data_store::DidWebvhDataStore;
use crate::keyring::{
    jwk::Jwk,
    keypair::{KeyPair, KeyPairing},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DidWebvhIdentifierError<StudioClientError: std::error::Error> {
    #[error("Failed to convert to JWK: {0}")]
    Jwk(#[from] crate::keyring::jwk::K256ToJwkError),
    #[error("Failed to parse body: {0}")]
    BodyParse(#[from] serde_json::Error),
    #[error("Failed to create identifier. response: {0}")]
    DidWebvhRequestFailed(String),
    #[error("Failed to send request: {0}")]
    DidWebvhHttpClient(StudioClientError),
    #[error("Failed to create identifier: {0}")]
    DidWebvhCreateLogEntryFailed(#[from] crate::did_webvh::domain::did_log_entry::DidLogEntryError),
    #[error("Failed to generate hash : {0}")]
    DidWebvhCreateGenerateHashFailed(
        #[from] crate::did_webvh::domain::crypto::crypto_utils::CryptoError,
    ),
}

#[derive(Debug, thiserror::Error)]
pub enum GetPublicKeyError {
    #[error("Failed to get public key: {0}")]
    PublicKeyNotFound(String),
    #[error("Failed to convert from JWK: {0}")]
    JwkToK256(#[from] crate::keyring::jwk::JwkToK256Error),
    #[error("Failed to convert from JWK: {0}")]
    JwkToX25519(#[from] crate::keyring::jwk::JwkToX25519Error),
}

fn get_key(key_type: &str, did_document: &DidDocument) -> Result<Jwk, GetPublicKeyError> {
    let did = &did_document.id;
    let public_key = did_document
        .verification_method
        .clone()
        .and_then(|vm| vm.into_iter().find(|pk| pk.id == key_type))
        .ok_or(GetPublicKeyError::PublicKeyNotFound(did.to_string()))?
        .public_key_jwk
        .ok_or(GetPublicKeyError::PublicKeyNotFound(did.to_string()))?;
    Ok(public_key)
}

pub fn get_sign_key(did_document: &DidDocument) -> Result<k256::PublicKey, GetPublicKeyError> {
    let public_key = get_key("#signingKey", did_document)?;
    Ok(public_key.try_into()?)
}

pub fn get_encrypt_key(
    did_document: &DidDocument,
) -> Result<x25519_dalek::PublicKey, GetPublicKeyError> {
    let public_key = get_key("#encryptionKey", did_document)?;
    Ok(public_key.try_into()?)
}

#[trait_variant::make(Send)]
pub trait DidWebvhService: Sync {
    type DidWebvhIdentifierError: std::error::Error + Send + Sync;
    async fn create_identifier(
        &self,
        path: &str,
        enable_prerotation: bool,
        keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhIdentifierError>;
    async fn update_identifier(
        &self,
        path: &str,
        keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhIdentifierError>;
    async fn deactivate_identifier(
        &self,
        path: &str,
        keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhIdentifierError>;
}

#[derive(Clone)]
pub struct DidWebvhServiceImpl<C: DidWebvhDataStore> {
    data_store: C,
}

impl<C: DidWebvhDataStore> DidWebvhServiceImpl<C> {
    pub fn new(data_store: C) -> Self {
        Self { data_store }
    }
}

impl<C> DidWebvhService for DidWebvhServiceImpl<C>
where
    C: DidWebvhDataStore + Send + Sync,
    C::Error: Send + Sync,
{
    type DidWebvhIdentifierError = DidWebvhIdentifierError<C::Error>;

    async fn create_identifier(
        &self,
        path: &str,
        enable_prerotation: bool,
        keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhIdentifierError> {
        let sign_key_jwk: Jwk = keyring.sign.get_public_key().try_into().map_err(|_| {
            DidWebvhIdentifierError::Jwk(crate::keyring::jwk::K256ToJwkError::PointsInvalid)
        })?;
        let encrypt_key_jwk: Jwk = keyring.encrypt.get_public_key().try_into().map_err(|_| {
            DidWebvhIdentifierError::Jwk(crate::keyring::jwk::K256ToJwkError::PointsInvalid)
        })?;

        let mut log_entry = DidLogEntry::new(&path)
            .map_err(|e| DidWebvhIdentifierError::DidWebvhCreateLogEntryFailed(e))?;
        let controller_key = keyring.update;
        let update_sec_key = controller_key.get_secret_key().to_bytes();
        let update_pub_key = multibase_encode(&controller_key.get_public_key().to_sec1_bytes());
        let update_keys = vec![update_pub_key.clone()];
        log_entry.parameters.update_keys = Some(update_keys);

        // if prerotation is enabled, add the prerotation key to the next_key_hashes
        if enable_prerotation {
            let prerotation_key =
                multibase_encode(&keyring.recovery.get_public_key().to_sec1_bytes());
            let prerotation_keys = vec![prerotation_key.clone()];
            log_entry = log_entry.calc_next_key_hash(&prerotation_keys)?;
        }
        let sign_verification_method = VerificationMethod {
            id: format!("{}#{}", log_entry.state.id, "signingKey".to_string()),
            r#type: "EcdsaSecp256k1VerificationKey2019".to_string(),
            controller: log_entry.state.id.clone(),
            public_key_jwk: Some(sign_key_jwk),
            blockchain_account_id: None,
            public_key_multibase: None,
        };
        let encrypt_verification_method = VerificationMethod {
            id: format!("{}#{}", log_entry.state.id, "encryptionKey".to_string()),
            r#type: "X25519KeyAgreementKey2019".to_string(),
            controller: log_entry.state.id.clone(),
            public_key_jwk: Some(encrypt_key_jwk),
            blockchain_account_id: None,
            public_key_multibase: None,
        };

        log_entry.state.verification_method =
            Some(vec![sign_verification_method, encrypt_verification_method]);
        let scid = log_entry.calc_entry_hash()?;
        log_entry.replace_placeholder_to_id(&scid)?;

        log_entry.generate_proof(&update_sec_key, &update_pub_key)?;

        let body =
            serde_json::to_string(&log_entry).map_err(|e| DidWebvhIdentifierError::BodyParse(e))?;

        let response = self
            .data_store
            .post(&path, &body)
            .await
            .map_err(|e| DidWebvhIdentifierError::DidWebvhRequestFailed(e.to_string()))?;
        if response.status_code.is_success() {
            let did_document: DidDocument = serde_json::from_str(&response.body)
                .map_err(|e| DidWebvhIdentifierError::BodyParse(e))?;
            Ok(did_document)
        } else {
            Err(DidWebvhIdentifierError::DidWebvhRequestFailed(
                response.body,
            ))
        }
    }

    async fn update_identifier(
        &self,
        _path: &str,
        _keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhIdentifierError> {
        unimplemented!()
    }

    async fn deactivate_identifier(
        &self,
        _path: &str,
        _keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhIdentifierError> {
        unimplemented!()
    }
}
