use crate::did_webvh::domain::crypto::crypto_utils::*;
use crate::did_webvh::domain::did::{Did, DidWebvh};
use crate::did_webvh::domain::did_document::DidDocument;
use crate::did_webvh::domain::did_log_entry::DidLogEntry;
use crate::did_webvh::infra::did_webvh_data_store::DidWebvhDataStore;
use crate::did_webvh::service::service_impl::DidWebvhServiceImpl;
use chrono::prelude::*;
use std::convert::TryInto;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DidWebvhResolverError<StudioClientError: std::error::Error> {
    #[error("Failed to resolve identifier: {0}")]
    ResolveIdentifier(String),
    #[error("Failed to convert to JWK: {0}")]
    Jwk(#[from] crate::keyring::jwk::K256ToJwkError),
    #[error("Failed to parse body: {0}")]
    BodyParse(#[from] serde_json::Error),
    #[error("Failed to parse log entry: {0}")]
    LogEntryParse(#[from] crate::did_webvh::domain::did_log_entry::DidLogEntryError),
    #[error("Failed to send request: {0}")]
    DidWebvhDataStore(StudioClientError),
    #[error("Failed to procesure of Did: {0}")]
    Did(#[from] crate::did_webvh::domain::did::DidError),
    #[error("Failed to procesure of DidWebvh: {0}")]
    DidWebvh(#[from] crate::did_webvh::domain::did::DidWebvhError),
    #[error("Failed to get identifier. response: {0}")]
    DidWebvhRequestFailed(String),
}

#[trait_variant::make(Send)]
pub trait DidWebvhResolverService: Sync {
    type DidWebvhResolverError: std::error::Error + Send + Sync;
    async fn resolve_identifier(
        &self,
        log_entries: Vec<DidLogEntry>,
    ) -> Result<DidDocument, Self::DidWebvhResolverError>;

    async fn get_identifier(
        &self,
        did: &Did,
    ) -> Result<Vec<DidLogEntry>, Self::DidWebvhResolverError>;
}

impl<C> DidWebvhResolverService for DidWebvhServiceImpl<C>
where
    C: DidWebvhDataStore + Send + Sync,
    C::Error: Send + Sync,
{
    type DidWebvhResolverError = DidWebvhResolverError<C::Error>;

    async fn resolve_identifier(
        &self,
        log_entries: Vec<DidLogEntry>,
    ) -> Result<DidDocument, Self::DidWebvhResolverError> {
        if log_entries.is_empty() {
            return Err(DidWebvhResolverError::DidWebvhRequestFailed(
                "No log entries found".to_string(),
            ));
        }

        let first_log_entry = log_entries.first().unwrap();
        let (id, hash) = first_log_entry.parse_version_id()?;
        if id != 1 {
            return Err(DidWebvhResolverError::ResolveIdentifier(
                "First log entry is not version 1".to_string(),
            ));
        }
        // check existence of update_keys in the first log entry, with check is_empty
        if first_log_entry.parameters.update_keys.is_some()
            && first_log_entry
                .parameters
                .update_keys
                .as_ref()
                .unwrap()
                .is_empty()
        {
            return Err(DidWebvhResolverError::ResolveIdentifier(
                "No update keys found".to_string(),
            ));
        }

        // check existence of proof in the first log entry
        if first_log_entry.proof.is_none() || first_log_entry.proof.as_ref().unwrap().is_empty() {
            return Err(DidWebvhResolverError::ResolveIdentifier(
                "No proof found".to_string(),
            ));
        }

        // verify proof section
        // contains verification_method in update_keys of first log entry.
        for proof in first_log_entry.proof.as_ref().unwrap() {
            let parsed_time = DateTime::parse_from_rfc3339(&proof.created).map_err(|_| {
                DidWebvhResolverError::ResolveIdentifier(
                    "Failed to parse proof created date".to_string(),
                )
            })?;
            if parsed_time > Utc::now() {
                return Err(DidWebvhResolverError::ResolveIdentifier(
                    "Proof created date is in the future".to_string(),
                ));
            }

            // remove prefix from verification_method, 'did:key:public_key#public_key' to
            // public_key, pickup last part of verification_method
            let public_key = proof.verification_method.split('#').last().ok_or(
                DidWebvhResolverError::ResolveIdentifier(
                    "Failed to split verification method".to_string(),
                ),
            )?;

            // check verification_method's public key is in update_keys
            if !first_log_entry
                .parameters
                .update_keys
                .as_ref()
                .unwrap()
                .contains(&public_key.to_string())
            {
                return Err(DidWebvhResolverError::ResolveIdentifier(
                    "Verification method not found in update keys".to_string(),
                ));
            }

            // verify signature
            // proof.verification_method is a public key, proof.proof_value is a signature
            let decoded_public_key = multibase_decode(public_key).map_err(|_| {
                DidWebvhResolverError::ResolveIdentifier("Failed to decode public key".to_string())
            })?;
            let decoded_proof_value = multibase_decode(&proof.proof_value).map_err(|_| {
                DidWebvhResolverError::ResolveIdentifier("Failed to decode proof value".to_string())
            })?;
            let jcs = serde_json_canonicalizer::to_string(&first_log_entry.remove_proof())
                .map_err(|_| {
                    DidWebvhResolverError::ResolveIdentifier(
                        "Failed to canonicalize log entry".to_string(),
                    )
                })?;

            if !verify_signature(jcs.as_bytes(), &decoded_proof_value, &decoded_public_key)
                .map_err(|e| DidWebvhResolverError::ResolveIdentifier(e.to_string()))?
            {
                return Err(DidWebvhResolverError::ResolveIdentifier(
                    "Failed to verify signature".to_string(),
                ));
            }
        }

        // verify entry hash
        // recalc scid and entry hash
        let mut replaced_hash_entry = first_log_entry.replace_to_placeholder()?;
        let calculated_scid = replaced_hash_entry.calc_entry_hash()?;
        replaced_hash_entry.replace_placeholder_to_id(&calculated_scid)?;

        let calculated_hash = replaced_hash_entry.calc_entry_hash()?;
        if calculated_hash != hash {
            return Err(DidWebvhResolverError::ResolveIdentifier(
                format!(
                    "Entry hash is not matched: calc {} != {}",
                    calculated_hash, hash
                )
                .to_string(),
            ));
        }

        let mut previous_version_number = id;
        let mut previous_entry = first_log_entry.clone();

        // verify the log entry iter start next entry
        for log_entry in log_entries.iter().skip(1) {
            // check version number is sequential
            let (id, hash) = log_entry.parse_version_id()?;
            if id != previous_version_number + 1 {
                return Err(DidWebvhResolverError::ResolveIdentifier(
                    "Log entry version is not sequential".to_string(),
                ));
            }

            // check version time is not in the future
            let current_version_time = DateTime::parse_from_rfc3339(&log_entry.version_time)
                .map_err(|_| {
                    DidWebvhResolverError::ResolveIdentifier(
                        "Failed to parse version time".to_string(),
                    )
                })?;
            let previous_version_time = DateTime::parse_from_rfc3339(&previous_entry.version_time)
                .map_err(|_| {
                    DidWebvhResolverError::ResolveIdentifier(
                        "Failed to parse version time".to_string(),
                    )
                })?;
            let current_time = Utc::now();
            if current_version_time > current_time || current_version_time > previous_version_time {
                return Err(DidWebvhResolverError::ResolveIdentifier(
                    "Log entry time is in the future".to_string(),
                ));
            }

            // verify vertion_id, current log entry's version_id is generated from previous log
            // entry's version_id
            let (_, previous_hash) = previous_entry.parse_version_id()?;
            let mut tmp_entry = log_entry.clone();
            tmp_entry.version_id = previous_hash;
            let recalculated_hash = tmp_entry.calc_entry_hash()?;
            if recalculated_hash != hash {
                return Err(DidWebvhResolverError::ResolveIdentifier(
                    "Version id is not matched".to_string(),
                ));
            }

            // verify prerotaion_keys, if next_key_hashes exists in previous log entry, compare
            // with calculated next_key_hashes from update_keys of current log entry.
            if let Some(mut previous_next_key_hashes) = previous_entry.parameters.next_key_hashes {
                let Some(current_update_keys) = log_entry.parameters.update_keys.as_ref() else {
                    return Err(DidWebvhResolverError::ResolveIdentifier(
                        "Current update keys not found".to_string(),
                    ));
                };
                let mut calculated_prerotation_keys =
                    log_entry.calc_next_key_hash(current_update_keys)?;
                // compare calculated_prerotation_keys and next_key_hashes
                calculated_prerotation_keys.sort();
                previous_next_key_hashes.sort();
                if calculated_prerotation_keys != previous_next_key_hashes {
                    return Err(DidWebvhResolverError::ResolveIdentifier(
                        "Prerotation keys are not matched".to_string(),
                    ));
                }
            }

            // verify proof section
            // if next_key_hashes exists in previous log entry, public key in proof section must
            // be in update_keys of current log entry. otherwise, public key in proof section must
            // be in update_keys of previous log entry.
            for proof in log_entry.proof.as_ref().unwrap() {
                let parsed_time = DateTime::parse_from_rfc3339(&proof.created).map_err(|_| {
                    DidWebvhResolverError::ResolveIdentifier(
                        "Failed to parse proof created date".to_string(),
                    )
                })?;
                if parsed_time > Utc::now() {
                    return Err(DidWebvhResolverError::ResolveIdentifier(
                        "Proof created date is in the future".to_string(),
                    ));
                }
                // remove prefix from verification_method, 'did:key:{public_key}'
                let verification_method: Did = proof
                    .verification_method
                    .split('#')
                    .next()
                    .ok_or(DidWebvhResolverError::ResolveIdentifier(
                        "Failed to split verification method".to_string(),
                    ))?
                    .parse::<Did>()?;
                let update_keys = if log_entry.parameters.next_key_hashes.is_some() {
                    log_entry.parameters.update_keys.as_ref().unwrap()
                } else {
                    previous_entry.parameters.update_keys.as_ref().unwrap()
                };
                if !update_keys.contains(&verification_method.get_method_specific_id().to_string())
                {
                    return Err(DidWebvhResolverError::ResolveIdentifier(
                        "Verification method not found in update keys".to_string(),
                    ));
                }
                // verify signature
                // proof.verification_method is a public key, proof.proof_value is a signature
                let public_key = verification_method.get_method_specific_id();
                let decoded_public_key = multibase_decode(public_key).map_err(|_| {
                    DidWebvhResolverError::ResolveIdentifier(
                        "Failed to decode public key".to_string(),
                    )
                })?;
                let decoded_proof_value = multibase_decode(&proof.proof_value).map_err(|_| {
                    DidWebvhResolverError::ResolveIdentifier(
                        "Failed to decode proof value".to_string(),
                    )
                })?;
                let jcs = serde_json_canonicalizer::to_string(&log_entry.remove_proof()).map_err(
                    |_| {
                        DidWebvhResolverError::ResolveIdentifier(
                            "Failed to canonicalize log entry".to_string(),
                        )
                    },
                )?;

                if verify_signature(jcs.as_bytes(), &decoded_proof_value, &decoded_public_key)
                    .map_err(|_| {
                        DidWebvhResolverError::ResolveIdentifier(
                            "Failed to verify signature".to_string(),
                        )
                    })?
                {
                    return Err(DidWebvhResolverError::ResolveIdentifier(
                        "Failed to verify signature".to_string(),
                    ));
                }
            }

            previous_version_number = id;
            previous_entry = log_entry.clone();
        }
        // Extract the latest DID Document from the last log entry.
        let last_log_entry = log_entries.last().unwrap();
        let did_document = last_log_entry.state.clone();
        Ok(did_document)
    }

    async fn get_identifier(
        &self,
        did: &Did,
    ) -> Result<Vec<DidLogEntry>, Self::DidWebvhResolverError> {
        let webvh_did: DidWebvh = did
            .clone()
            .try_into()
            .map_err(DidWebvhResolverError::DidWebvh)?;
        let converted_did = webvh_did.did_to_https();

        Ok(self
            .data_store
            .get(&converted_did)
            .await
            .map_err(DidWebvhResolverError::DidWebvhDataStore)?)
    }
}
