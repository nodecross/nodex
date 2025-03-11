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
pub enum ResolveIdentifierError {
    #[error("First log entry is not version 1")]
    InvalidFirstVersion,
    #[error("Log entry time is in the future")]
    FutureTime,
    #[error("No update keys found")]
    NoUpdateKeys,
    #[error("Log entry version is not sequential")]
    InvalidVersionSequence,
    #[error("Prerotation keys are not matched")]
    InvalidPrerotationKey,
    #[error("Entry hash is not matched: calc {found:?} != {expected:?}")]
    InvalidEntryHash { expected: String, found: String },
    #[error("No proof found")]
    NoProof,
    #[error("Proof created date is in the future")]
    FutureProofTime,
    #[error("Failed to parse proof created date")]
    InvalidProof,
    #[error("Failed to split verification method")]
    SplitVerficationMethod,
    #[error("Verification method not found in update keys")]
    NoVerificationMethod,
    #[error("Failed to decode public key")]
    DecodePubilcKey,
    #[error("Failed to decode proof value")]
    DecodeProof,
    #[error("Failed to canonicalize log entry")]
    Canonicalize,
    #[error("Failed to verify signature")]
    VerifySignature,
    #[error("Failed to procesure of Did: {0}")]
    Did(#[from] crate::did_webvh::domain::did::DidError),
    #[error("Failed to parse version time: {0}")]
    DateParse(#[from] chrono::ParseError),
    #[error("Failed to parse log entry: {0}")]
    LogEntryParse(#[from] crate::did_webvh::domain::did_log_entry::DidLogEntryError),
}
#[derive(Debug, Error)]
pub enum DidWebvhResolverError<StudioClientError: std::error::Error> {
    #[error("Failed to resolve identifier: {0}")]
    ResolveIdentifier(#[from] ResolveIdentifierError),
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
    #[error("Failed to convert did to url")]
    ConvertDid,
}

#[trait_variant::make(Send)]
pub trait DidWebvhResolverService: Sync {
    type DidWebvhResolverError: std::error::Error + Send + Sync;
    async fn resolve_identifier(
        &mut self,
        did: &Did,
    ) -> Result<Option<DidDocument>, Self::DidWebvhResolverError>;
}

// verify proof section
// if next_key_hashes exists in previous log entry, public key in proof section must
// be in update_keys of current log entry. otherwise, public key in proof section must
// be in update_keys of previous log entry.
fn verify_proofs(
    update_keys: &[String],
    log_entry: &DidLogEntry,
) -> Result<(), ResolveIdentifierError> {
    // check existence of proof in log entry
    let proofs = match log_entry.proof.as_deref() {
        None | Some(&[]) => return Err(ResolveIdentifierError::NoProof),
        Some(proofs) => proofs,
    };
    for proof in proofs {
        let parsed_time = DateTime::parse_from_rfc3339(&proof.created)
            .map_err(ResolveIdentifierError::DateParse)?;
        if parsed_time > Utc::now() {
            return Err(ResolveIdentifierError::FutureProofTime);
        }
        // remove prefix from verification_method, 'did:key:{public_key}'
        let verification_method: Did = proof
            .verification_method
            .split('#')
            .next()
            .ok_or(ResolveIdentifierError::SplitVerficationMethod)?
            .parse::<Did>()?;
        if !update_keys.contains(&verification_method.get_method_specific_id().to_string()) {
            return Err(ResolveIdentifierError::NoVerificationMethod);
        }
        // verify signature
        // proof.verification_method is a public key, proof.proof_value is a signature
        let public_key = verification_method.get_method_specific_id();
        let decoded_public_key =
            multibase_decode(public_key).map_err(|_| ResolveIdentifierError::DecodePubilcKey)?;
        let decoded_proof_value = multibase_decode(&proof.proof_value)
            .map_err(|_| ResolveIdentifierError::DecodeProof)?;
        let jcs = serde_json_canonicalizer::to_string(&log_entry.remove_proof())
            .map_err(|_| ResolveIdentifierError::Canonicalize)?;

        if !verify_signature(jcs.as_bytes(), &decoded_proof_value, &decoded_public_key)
            .map_err(|_| ResolveIdentifierError::VerifySignature)?
        {
            return Err(ResolveIdentifierError::VerifySignature);
        }
    }
    Ok(())
}

pub fn verify_entries(
    log_entries: &[DidLogEntry],
) -> Result<Option<DidDocument>, ResolveIdentifierError> {
    let (first_log_entry, rest_log_entries) = match log_entries.split_first() {
        Some(log_entries) => log_entries,
        None => return Ok(None),
    };
    let current_time = Utc::now();

    let (id, hash) = first_log_entry.parse_version_id()?;
    if id != 1 {
        return Err(ResolveIdentifierError::InvalidFirstVersion);
    }
    // check version time is not in the future
    let current_version_time = DateTime::parse_from_rfc3339(&first_log_entry.version_time)
        .map_err(ResolveIdentifierError::DateParse)?;
    if current_version_time > current_time {
        return Err(ResolveIdentifierError::FutureTime);
    }
    // check existence of update_keys in the first log entry, with check is_empty
    let mut update_keys = match first_log_entry.parameters.update_keys.as_deref() {
        None | Some(&[]) => return Err(ResolveIdentifierError::NoUpdateKeys),
        Some(update_keys) => update_keys,
    };

    verify_proofs(update_keys, first_log_entry)?;

    // verify entry hash
    // recalc scid and entry hash
    let mut replaced_hash_entry = first_log_entry.replace_to_placeholder()?;
    let calculated_scid = replaced_hash_entry.calc_entry_hash()?;
    replaced_hash_entry.replace_placeholder_to_id(&calculated_scid)?;

    let calculated_hash = replaced_hash_entry.calc_entry_hash()?;
    if calculated_hash != hash {
        return Err(ResolveIdentifierError::InvalidEntryHash {
            expected: hash,
            found: calculated_hash,
        });
    }

    let mut previous_version_number = id;
    let mut previous_entry = first_log_entry.clone();

    // verify the log entry iter start next entry
    for log_entry in rest_log_entries.iter() {
        // check version number is sequential
        let (id, hash) = log_entry.parse_version_id()?;
        if id != previous_version_number + 1 {
            return Err(ResolveIdentifierError::InvalidVersionSequence);
        }

        // check version time is not in the future
        let current_version_time = DateTime::parse_from_rfc3339(&log_entry.version_time)
            .map_err(ResolveIdentifierError::DateParse)?;
        if current_version_time > current_time {
            return Err(ResolveIdentifierError::FutureTime);
        }
        let previous_version_time = DateTime::parse_from_rfc3339(&previous_entry.version_time)
            .map_err(ResolveIdentifierError::DateParse)?;
        if current_version_time > previous_version_time {
            return Err(ResolveIdentifierError::FutureTime);
        }

        // verify vertion_id, current log entry's version_id is generated from previous log
        // entry's version_id
        let recalculated_hash = {
            let (_, previous_hash) = previous_entry.parse_version_id()?;
            let mut tmp_entry = log_entry.clone();
            tmp_entry.version_id = previous_hash;
            tmp_entry.calc_entry_hash()?
        };

        if recalculated_hash != hash {
            return Err(ResolveIdentifierError::InvalidEntryHash {
                expected: hash,
                found: recalculated_hash,
            });
        }

        // verify prerotaion_keys, if next_key_hashes exists in previous log entry, compare
        // with calculated next_key_hashes from update_keys of current log entry.
        if let Some(mut previous_next_key_hashes) = previous_entry.parameters.next_key_hashes {
            let Some(current_update_keys) = log_entry.parameters.update_keys.as_ref() else {
                return Err(ResolveIdentifierError::NoUpdateKeys);
            };
            let mut calculated_prerotation_keys =
                log_entry.calc_next_key_hash(current_update_keys)?;
            // compare calculated_prerotation_keys and next_key_hashes
            calculated_prerotation_keys.sort();
            previous_next_key_hashes.sort();
            if calculated_prerotation_keys != previous_next_key_hashes {
                return Err(ResolveIdentifierError::InvalidPrerotationKey);
            }
            update_keys = current_update_keys;
        }

        verify_proofs(update_keys, log_entry)?;

        previous_version_number = id;
        previous_entry = log_entry.clone();
    }
    // Extract the latest DID Document from the last log entry.
    let last_log_entry = previous_entry;
    let did_document = last_log_entry.state;
    Ok(Some(did_document))
}

impl<C> DidWebvhResolverService for DidWebvhServiceImpl<C>
where
    C: DidWebvhDataStore + Send + Sync,
    C::Error: Send + Sync,
{
    type DidWebvhResolverError = DidWebvhResolverError<C::Error>;

    async fn resolve_identifier(
        &mut self,
        did: &Did,
    ) -> Result<Option<DidDocument>, Self::DidWebvhResolverError> {
        let webvh_did: DidWebvh = did.clone().try_into()?;
        let converted_did = webvh_did
            .to_url_without_method()
            .ok_or(DidWebvhResolverError::ConvertDid)?;
        let method = if self.use_https { "https" } else { "http" };
        let converted_did = format!("{}://{}", method, converted_did);

        let entries = self
            .data_store
            .get(&converted_did)
            .await
            .map_err(DidWebvhResolverError::DidWebvhDataStore)?;
        Ok(verify_entries(&entries)?)
    }
}
