use crate::did_webvh::domain::crypto::crypto_utils::multibase_encode;
use crate::did_webvh::domain::did_document::DidDocument;
use crate::did_webvh::domain::did_log_entry::DidLogEntry;
use crate::did_webvh::infra::did_webvh_data_store::DidWebvhDataStore;
use crate::did_webvh::service::service_impl::DidWebvhServiceImpl;
use crate::keyring::keypair::{KeyPair, KeyPairing};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerateIdentifierError {
    #[error("Failed to get public key: {0}")]
    PublicKey(#[from] crate::keyring::jwk::K256ToJwkError),
    #[error("Failed to generate hash: {0}")]
    CreateLogEntry(#[from] crate::did_webvh::domain::did_log_entry::DidLogEntryError),
}

#[derive(Debug, Error)]
pub enum DidWebvhControllerError<StudioClientError: std::error::Error> {
    #[error("Failed to create identifier: {0}")]
    GenerateIdentifier(#[from] GenerateIdentifierError),
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

#[trait_variant::make(Send)]
pub trait DidWebvhControllerService: Sync {
    type DidWebvhControllerError: std::error::Error + Send + Sync;
    async fn create_identifier(
        &mut self,
        path: &str,
        enable_prerotation: bool,
        keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError>;
    async fn update_identifier(
        &mut self,
        path: &str,
        keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError>;
    async fn deactivate_identifier(
        &mut self,
        path: &str,
        keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError>;
}

pub fn generate_log_entry(
    path: &str,
    enable_prerotation: bool,
    keyring: KeyPairing,
) -> Result<Vec<DidLogEntry>, GenerateIdentifierError> {
    let mut log_entry = DidLogEntry::new(path)?;
    let vms = keyring
        .to_verification_methods(&log_entry.state.id)
        .map_err(GenerateIdentifierError::PublicKey)?;

    let update_keypair = keyring.didwebvh_update;
    let update_sec_key = update_keypair.get_secret_key().to_bytes();
    let update_pub_key = multibase_encode(&update_keypair.get_public_key().to_bytes());
    let update_keys = vec![update_pub_key.clone()];
    log_entry.parameters.update_keys = Some(update_keys);

    // if prerotation is enabled, add the prerotation key to the next_key_hashes
    if enable_prerotation {
        let prerotation_pub_key =
            multibase_encode(&keyring.didwebvh_recovery.get_public_key().to_bytes());
        let prerotation_keys = vec![prerotation_pub_key];
        let next_key_hases = log_entry.calc_next_key_hash(&prerotation_keys)?;
        log_entry.parameters.next_key_hashes = Some(next_key_hases);
    }

    for vm in vms {
        log_entry.state.add_verification_method(vm);
    }

    let scid = log_entry.calc_entry_hash()?;
    log_entry.replace_placeholder_to_id(&scid)?;
    let first_entry_hash = log_entry.calc_entry_hash()?;
    log_entry.version_id = format!("1-{}", first_entry_hash);

    log_entry.generate_proof(&update_sec_key, &update_pub_key)?;

    Ok(vec![log_entry])
}

impl<C> DidWebvhControllerService for DidWebvhServiceImpl<C>
where
    C: DidWebvhDataStore + Send + Sync,
    C::Error: Send + Sync,
{
    type DidWebvhControllerError = DidWebvhControllerError<C::Error>;

    async fn create_identifier(
        &mut self,
        path: &str,
        enable_prerotation: bool,
        keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError> {
        let entries = generate_log_entry(path, enable_prerotation, keyring)?;
        let response = self
            .data_store
            .create(path, &entries)
            .await
            .map_err(|e| DidWebvhControllerError::DidWebvhRequestFailed(e.to_string()))?;
        Ok(response)
    }

    async fn update_identifier(
        &mut self,
        _path: &str,
        _keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError> {
        unimplemented!()
    }

    async fn deactivate_identifier(
        &mut self,
        _path: &str,
        _keyring: KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError> {
        unimplemented!()
    }
}
