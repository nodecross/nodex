use crate::did_webvh::domain::crypto::crypto_utils::multibase_encode;
use crate::did_webvh::domain::did::DidWebvh;
use crate::did_webvh::domain::did_document::DidDocument;
use crate::did_webvh::domain::did_log_entry::DidLogEntry;
use crate::did_webvh::infra::did_webvh_data_store::DidWebvhDataStore;
use crate::did_webvh::service::service_impl::DidWebvhServiceImpl;
use crate::keyring::keypair::{KeyPair, KeyPairing};
use crate::rand_core::OsRng;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerateIdentifierError {
    #[error("Failed to get public key: {0}")]
    PublicKey(#[from] crate::keyring::jwk::K256ToJwkError),
    #[error("Failed to generate hash: {0}")]
    CreateLogEntry(#[from] crate::did_webvh::domain::did_log_entry::DidLogEntryError),
}

#[derive(Debug, Error)]
pub enum UpdateIdentifierError {
    #[error("No entries found")]
    NoEntries,
    #[error("Invalid Prerotation Keys")]
    PrerotationKeys,
    #[error("Invalid DId Method")]
    ConvertDidMethod(#[from] crate::did_webvh::domain::did::DidWebvhError),
    #[error("Failed to verify log entry: {0}")]
    LogEntry(#[from] crate::did_webvh::domain::did_log_entry::DidLogEntryError),
    #[error("Failed to convert key to verification method: {0}")]
    PublicKey(#[from] crate::keyring::jwk::K256ToJwkError),
}

#[derive(Debug, Error)]
pub enum DidWebvhControllerError<StudioClientError: std::error::Error> {
    #[error("Failed to create identifier: {0}")]
    GenerateIdentifier(#[from] GenerateIdentifierError),
    #[error("Failed to update identifier: {0}")]
    UpdateIdentifier(#[from] UpdateIdentifierError),
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
        log_entries: &Vec<DidLogEntry>,
        enable_prerotation: bool,
        keyring: &mut KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError>;
    async fn deactivate_identifier(
        &mut self,
        log_entries: &Vec<DidLogEntry>,
        keyring: &mut KeyPairing,
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

pub fn append_new_entry(
    log_entries: &Vec<DidLogEntry>,
    enable_prerotation: bool,
    keyring: &mut KeyPairing,
) -> Result<Vec<DidLogEntry>, UpdateIdentifierError> {
    let latest_entry = if log_entries.is_empty() {
        return Err(UpdateIdentifierError::NoEntries);
    } else {
        log_entries.last().unwrap().clone()
    };
    let (previous_id, _) = latest_entry.parse_version_id()?;

    // Generate a new log entry based on the existing log entries
    // TODO: If we need to consider updates that change the URL (domain) included in the DID, this part will need to be modified
    // The current implementation assumes that the DID itself included in the existing log entry will not change
    let mut new_entry = latest_entry.generate_next_log_entry()?;

    let rotated_keying = keyring.rotate_keypair(OsRng);
    let update_keypair = rotated_keying.didwebvh_update.clone();
    let update_sec_key = update_keypair.get_secret_key().to_bytes();
    let update_pub_key = multibase_encode(&update_keypair.get_public_key().to_bytes());

    let update_keys = vec![update_pub_key.clone()];

    // verify prerotaion_keys, if next_key_hashes exists in previous log entry,
    // compare with calculated next_key_hashes from update_keys of current log entry.
    if let Some(mut previous_next_key_hashes) = latest_entry.parameters.next_key_hashes {
        // compare calculated_prerotation_keys and next_key_hashes
        let mut calc_key_hash_from_current_update_key =
            new_entry.calc_next_key_hash(&update_keys)?;
        calc_key_hash_from_current_update_key.sort();
        previous_next_key_hashes.sort();
        if calc_key_hash_from_current_update_key != previous_next_key_hashes {
            return Err(UpdateIdentifierError::PrerotationKeys);
        }
    }

    new_entry.parameters.update_keys = Some(update_keys);

    if enable_prerotation {
        let next_keypair = rotated_keying.didwebvh_recovery.clone();
        let prerotation_pub_key = multibase_encode(&next_keypair.get_public_key().to_bytes());
        let prerotation_keys = vec![prerotation_pub_key];
        let next_key_hases = new_entry.calc_next_key_hash(&prerotation_keys)?;
        new_entry.parameters.next_key_hashes = Some(next_key_hases);
    }
    let vms = &rotated_keying.to_verification_methods(&latest_entry.state.id)?;
    new_entry.state.verification_method = None;
    for vm in vms {
        new_entry.state.add_verification_method(vm.clone());
    }

    let entry_hash = new_entry.calc_entry_hash()?;
    let current_id = previous_id + 1;
    new_entry.version_id = format!("{}-{}", current_id, entry_hash);

    new_entry.generate_proof(&update_sec_key, &update_pub_key)?;

    let mut new_log_entries = log_entries.clone();
    new_log_entries.push(new_entry);

    Ok(new_log_entries)
}

pub fn append_deactivation_entry(
    log_entries: &Vec<DidLogEntry>,
    keyring: &mut KeyPairing,
) -> Result<Vec<DidLogEntry>, UpdateIdentifierError> {
    let latest_entry = if log_entries.is_empty() {
        return Err(UpdateIdentifierError::NoEntries);
    } else {
        log_entries.last().unwrap().clone()
    };
    let (previous_id, _) = latest_entry.parse_version_id()?;

    // Generate a new log entry based on the existing log entries
    // TODO: If we need to consider updates that change the URL (domain) included in the DID, this part will need to be modified
    // The current implementation assumes that the DID itself included in the existing log entry will not change
    let mut new_entry = latest_entry.generate_next_log_entry()?;

    let rotated_keying = keyring.rotate_keypair(OsRng);
    let update_keypair = rotated_keying.didwebvh_update.clone();
    let update_sec_key = update_keypair.get_secret_key().to_bytes();
    let update_pub_key = multibase_encode(&update_keypair.get_public_key().to_bytes());

    let update_keys = vec![update_pub_key.clone()];

    // verify prerotaion_keys, if next_key_hashes exists in previous log entry,
    // compare with calculated next_key_hashes from update_keys of current log entry.
    if let Some(mut previous_next_key_hashes) = latest_entry.parameters.next_key_hashes {
        // compare calculated_prerotation_keys and next_key_hashes
        let mut calc_key_hash_from_current_update_key =
            new_entry.calc_next_key_hash(&update_keys)?;
        calc_key_hash_from_current_update_key.sort();
        previous_next_key_hashes.sort();
        if calc_key_hash_from_current_update_key != previous_next_key_hashes {
            return Err(UpdateIdentifierError::PrerotationKeys);
        }
    }

    // Do not add update_keys to the deactivation entry
    new_entry.parameters.update_keys = None;
    // Similarly, do not add next key hashes to the deactivation entry
    new_entry.parameters.next_key_hashes = None;

    let mut deactivate_entry = new_entry.deactivate();
    let vms = &rotated_keying.to_verification_methods(&deactivate_entry.state.id)?;
    deactivate_entry.state.verification_method = None;
    for vm in vms {
        deactivate_entry.state.add_verification_method(vm.clone());
    }
    deactivate_entry.state = deactivate_entry.state.deactivate();

    let entry_hash = deactivate_entry.calc_entry_hash()?;
    let current_id = previous_id + 1;
    deactivate_entry.version_id = format!("{}-{}", current_id, entry_hash);

    deactivate_entry.generate_proof(&update_sec_key, &update_pub_key)?;

    let mut new_log_entries = log_entries.clone();
    new_log_entries.push(deactivate_entry);

    Ok(new_log_entries)
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
        log_entries: &Vec<DidLogEntry>,
        enable_prerotation: bool,
        keyring: &mut KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError> {
        let new_log_entries = append_new_entry(log_entries, enable_prerotation, keyring)?;
        let Some(new_entry) = new_log_entries.last() else {
            return Err(DidWebvhControllerError::UpdateIdentifier(
                UpdateIdentifierError::NoEntries,
            ));
        };
        let did = DidWebvh::try_from(new_entry.state.id.clone()).map_err(|e| {
            DidWebvhControllerError::UpdateIdentifier(UpdateIdentifierError::ConvertDidMethod(e))
        })?;
        let uri = did.get_uri();
        let response = self
            .data_store
            .update(&uri, &new_log_entries)
            .await
            .map_err(|e| DidWebvhControllerError::DidWebvhRequestFailed(e.to_string()))?;

        Ok(response)
    }

    async fn deactivate_identifier(
        &mut self,
        log_entries: &Vec<DidLogEntry>,
        keyring: &mut KeyPairing,
    ) -> Result<DidDocument, Self::DidWebvhControllerError> {
        let deactivate_log_entries = append_deactivation_entry(log_entries, keyring)?;
        let Some(deactivate_entry) = deactivate_log_entries.last() else {
            return Err(DidWebvhControllerError::UpdateIdentifier(
                UpdateIdentifierError::NoEntries,
            ));
        };
        let did = DidWebvh::try_from(deactivate_entry.state.id.clone()).map_err(|e| {
            DidWebvhControllerError::UpdateIdentifier(UpdateIdentifierError::ConvertDidMethod(e))
        })?;
        let uri = did.get_uri();
        let response = self
            .data_store
            .deactivate(&uri, &deactivate_log_entries)
            .await
            .map_err(|e| DidWebvhControllerError::DidWebvhRequestFailed(e.to_string()))?;

        Ok(response)
    }
}
