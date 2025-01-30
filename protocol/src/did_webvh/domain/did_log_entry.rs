use super::crypto::hash::generate_multihash_with_base58_encode;
use super::did::{Did, DidWebvh};
use super::did_document::DidDocument;
use chrono::DateTime;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json_canonicalizer;
use validator::{Validate, ValidationError};

const WEBVH_DID_METHOD: &str = "did:webvh:0.5";
const WEBVH_DID_CRYPTO_SUITE: &str = "eddsa-jcs-2022";
const WEBVH_DID_SCID_PLACEHOLDER: &str = "{SCID}";

#[derive(Debug, PartialEq, Eq)]
pub enum DidLogEntryError {
    InvalidVersionId,
    InvalidVersionTime,
    InvalidParameters,
    InvalidState,
    InvalidProof,
    InvalidFormat,
    FaildMultihash,
    NotFound,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidLogEntry {
    #[validate(custom(function = "verify_version_id"))]
    pub version_id: String,
    #[validate(custom(function = "verify_version_time"))]
    pub version_time: String,
    pub parameters: Parameters,
    pub state: DidDocument,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<Vec<Proof>>,
}

fn verify_version_id(version_id: &str) -> Result<(), ValidationError> {
    // format is <version_number>-<base58(multihash(jsc(did_log_entry without proof)))>
    let re = Regex::new(r"^\d+-[1-9A-HJ-NP-Za-km-z]+$").unwrap();
    if !re.is_match(version_id) {
        return Err(ValidationError::new(
            "Version ID must start with a number followed by a hyphen",
        ));
    }
    Ok(())
}
fn verify_version_time(version_time: &str) -> Result<(), ValidationError> {
    // The version_time MUST be a valid UTC ISO8601 date/time string. let parsed_time = DateTime::parse_from_rfc3339(version_time);
    let parsed_time = DateTime::parse_from_rfc3339(version_time);
    if parsed_time.is_err() {
        return Err(ValidationError::new("Invalid version time"));
    }
    let current_time = chrono::Utc::now();
    let version_time = parsed_time.unwrap();

    // version_time MUST be in the past or current time
    if version_time > current_time {
        return Err(ValidationError::new("Version time is in the future"));
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "verify_next_key_hashes"))]
    pub next_key_hashes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "verify_method"))]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deactivate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<WitnessConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
}

fn verify_next_key_hashes(next_key_hashes: &Vec<String>) -> Result<(), ValidationError> {
    if next_key_hashes.is_empty() {
        return Err(ValidationError::new("Next key hashes must not be empty"));
    }
    Ok(())
}

fn verify_method(method: &str) -> Result<(), ValidationError> {
    if method != WEBVH_DID_METHOD {
        return Err(ValidationError::new("Invalid method"));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WitnessConfig {
    pub threshold: u32,
    pub witnesses: Vec<Witness>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Witness {
    #[validate(custom(function = "verify_witness_id"))]
    pub id: Did,
    pub weight: u32,
}

fn verify_witness_id(id: &Did) -> Result<(), ValidationError> {
    if id.get_method() != "webvh" {
        return Err(ValidationError::new("Invalid witness id"));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Proof {
    #[serde(rename = "type")]
    #[validate(custom(function = "verify_proof_type"))]
    pub r#type: String,
    #[validate(custom(function = "verify_proof_cryptosuite"))]
    pub cryptosuite: String,
    #[validate(custom(function = "verify_proof_verification_method"))]
    pub verification_method: String,
    #[validate(custom(function = "verify_proof_created"))]
    pub created: String,
    #[validate(custom(function = "verify_proof_proof_purpose"))]
    pub proof_purpose: String,
    pub proof_value: String,
}

fn verify_proof_type(proof_type: &str) -> Result<(), ValidationError> {
    if proof_type != "DataIntegrityProof" {
        return Err(ValidationError::new("Invalid proof type"));
    }
    Ok(())
}

fn verify_proof_cryptosuite(cryptosuite: &str) -> Result<(), ValidationError> {
    if cryptosuite != WEBVH_DID_CRYPTO_SUITE {
        return Err(ValidationError::new("Invalid cryptosuite"));
    }
    Ok(())
}

fn verify_proof_proof_purpose(proof_purpose: &str) -> Result<(), ValidationError> {
    if proof_purpose != "authentication" {
        return Err(ValidationError::new("Invalid proof purpose"));
    }
    Ok(())
}

fn verify_proof_verification_method(verification_method: &str) -> Result<(), ValidationError> {
    let parts: Vec<&str> = verification_method.split('#').collect();
    if parts.len() != 2 {
        return Err(ValidationError::new("Invalid verification method"));
    }
    let did = parts[0]
        .parse::<Did>()
        .expect("Invalid verification method");
    if did.get_method() != "key" {
        return Err(ValidationError::new("Invalid verification method"));
    }
    Ok(())
}

fn verify_proof_created(created: &str) -> Result<(), ValidationError> {
    let parsed_time = DateTime::parse_from_rfc3339(created);
    if parsed_time.is_err() {
        return Err(ValidationError::new("Invalid created time"));
    }
    Ok(())
}

impl DidLogEntry {
    // Create a new DIDLogEntry, with scid placeholder
    pub fn new(uri: &str) -> Result<Self, DidLogEntryError> {
        let did = DidWebvh::new(WEBVH_DID_SCID_PLACEHOLDER, uri)
            .map_err(|_| DidLogEntryError::InvalidFormat)?;
        let state = DidDocument::new(did.get_did().clone());
        let version_id = WEBVH_DID_SCID_PLACEHOLDER.to_string();
        let version_time = chrono::Utc::now().to_rfc3339();
        let parameters = Parameters {
            portable: None,
            update_keys: None,
            next_key_hashes: None,
            method: Some(WEBVH_DID_METHOD.to_string()),
            scid: Some(WEBVH_DID_SCID_PLACEHOLDER.to_string()),
            deactivate: None,
            witness: None,
            ttl: None,
        };
        Ok(Self {
            version_id,
            version_time,
            parameters,
            state,
            proof: None,
        })
    }

    // create a new DIDLogEntry from current entry.
    pub fn generate_next_log_entry(&self) -> Result<Self, DidLogEntryError> {
        let (_, next_version_id) = self.parse_verion_id()?;
        let version_time = chrono::Utc::now().to_rfc3339();
        Ok(Self {
            version_id: next_version_id,
            version_time,
            parameters: self.parameters.clone(),
            state: self.state.clone(),
            proof: None,
        })
    }

    pub fn parse_verion_id(&self) -> Result<(u32, String), DidLogEntryError> {
        let parts: Vec<&str> = self.version_id.split('-').collect();
        if parts.len() != 2 {
            return Err(DidLogEntryError::InvalidVersionId);
        }
        let version_number = parts[0].parse::<u32>();
        if version_number.is_err() {
            return Err(DidLogEntryError::InvalidVersionId);
        }
        Ok((version_number.unwrap(), parts[1].to_string()))
    }

    pub fn replace_to_scid_placeholder(&self) -> Result<DidLogEntry, DidLogEntryError> {
        let scid_placeholder = "{SCID}";
        let mut entry = self.clone();
        entry.parameters.scid = Some(scid_placeholder.to_string());
        entry.version_id = scid_placeholder.to_string();
        let did = DidWebvh::try_from(entry.state.id.clone())
            .map_err(|_| DidLogEntryError::InvalidState)?
            .replace_scid(scid_placeholder);
        entry.state.id = did.into();
        Ok(entry)
    }

    pub fn generate_proof(
        &self,
        public_key: &str,
        proof_purpose: &str,
        proof_value: &str,
    ) -> Proof {
        let key = format!("did:key:{}#{}", public_key, public_key);
        Proof {
            r#type: "DataIntegrityProof".to_string(),
            cryptosuite: WEBVH_DID_CRYPTO_SUITE.to_string(),
            verification_method: key,
            created: chrono::Utc::now().to_rfc3339(),
            proof_purpose: proof_purpose.to_string(),
            proof_value: proof_value.to_string(),
        }
    }

    // calculate the entry hash
    // The first entry, LogEntry with SCID placeholder can use this function to compute the SCID.
    // Otherwise, just be calculated as an EntryHash
    pub fn calc_entry_hash(&self) -> Result<String, DidLogEntryError> {
        let mut entry = self.clone();
        if entry.proof.is_some() {
            entry.proof = None;
        }
        let jcs = serde_json_canonicalizer::to_string(&entry)
            .map_err(|_| DidLogEntryError::InvalidFormat)?;
        generate_multihash_with_base58_encode(jcs.as_bytes())
            .map_err(|_| DidLogEntryError::FaildMultihash)
    }

    // calculate the next key hashes by the Update Keys from the previous entry.
    pub fn calc_next_key_hash(&self, keys: &Vec<String>) -> Result<Self, DidLogEntryError> {
        let generate_hashed_keys = |keys: &Vec<String>| {
            keys.iter()
                .map(|key| {
                    generate_multihash_with_base58_encode(key.as_bytes())
                        .map_err(|_| DidLogEntryError::FaildMultihash)
                })
                .collect::<Result<Vec<String>, DidLogEntryError>>()
        };
        let mut entry = self.clone();
        entry.parameters.next_key_hashes = Some(generate_hashed_keys(keys)?);
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{"versionId": "1-QmRD52wqs942kZ2gs7UU9QmaopvqnMziqB4qgFDYsapCT9", "versionTime": "2024-10-01T22:13:49Z", "parameters": {"updateKeys": ["z6Mkkr7iopdwZUgE87YaypKXSuBTsT6C7TyaUABmnHMuqmTY"], "nextKeyHashes": ["QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr"], "method": "did:webvh:0.5", "scid": "QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU"}, "state": {"@context": ["https://www.w3.org/ns/did/v1"], "id": "did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example"}, "proof": [{"type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "verificationMethod": "did:key:z6Mkkr7iopdwZUgE87YaypKXSuBTsT6C7TyaUABmnHMuqmTY#z6Mkkr7iopdwZUgE87YaypKXSuBTsT6C7TyaUABmnHMuqmTY", "created": "2024-10-01T22:13:49Z", "proofPurpose": "authentication", "proofValue": "z3HXr9s1oJ8Uf81zdVUeN4a5oEDJHH46kFTgZ6uEruN6ZCZucTFmJvezY8hCLPjKBpF2rJVwHpdVWE2x621xTGvpK"}]}"#;

    const JSONL: &str = r#"{"versionId": "1-QmRD52wqs942kZ2gs7UU9QmaopvqnMziqB4qgFDYsapCT9", "versionTime": "2024-10-01T22:13:49Z", "parameters": {"prerotation": true, "updateKeys": ["z6Mkkr7iopdwZUgE87YaypKXSuBTsT6C7TyaUABmnHMuqmTY"], "nextKeyHashes": ["QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr"], "method": "did:webvh:0.5", "scid": "QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU"}, "state": {"@context": ["https://www.w3.org/ns/did/v1"], "id": "did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example"}, "proof": [{"type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "verificationMethod": "did:key:z6Mkkr7iopdwZUgE87YaypKXSuBTsT6C7TyaUABmnHMuqmTY#z6Mkkr7iopdwZUgE87YaypKXSuBTsT6C7TyaUABmnHMuqmTY", "created": "2024-10-01T22:13:49Z", "proofPurpose": "authentication", "proofValue": "z3HXr9s1oJ8Uf81zdVUeN4a5oEDJHH46kFTgZ6uEruN6ZCZucTFmJvezY8hCLPjKBpF2rJVwHpdVWE2x621xTGvpK"}]}
        {"versionId": "2-QmV9Kh7GTCWBhxeKoZfWGC1QpJh1oQNhkf34RjpDZjsRhu", "versionTime": "2024-10-01T22:13:49Z", "parameters": {"updateKeys": ["z6MkoSFjacZb7R82htx8n1AkpgLQWR7CA6rigsc2VH9acLuF"], "nextKeyHashes": ["QmTCxXN3Wyo2PEqnyn5zfgW2iPYZ9gijyeTp6TDxQAA6Xw"]}, "state": {"@context": ["https://www.w3.org/ns/did/v1"], "id": "did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example"}, "proof": [{"type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "verificationMethod": "did:key:z6Mkkr7iopdwZUgE87YaypKXSuBTsT6C7TyaUABmnHMuqmTY#z6Mkkr7iopdwZUgE87YaypKXSuBTsT6C7TyaUABmnHMuqmTY", "created": "2024-10-01T22:13:49Z", "proofPurpose": "authentication", "proofValue": "ziBh1y9Uf4xB1VWDc8YyZSGMWLLwE8CV4RWz9iT6bHRnbW8q8MndUuWLivBydNeBfX8qjKPcMX9vGTFyUWUm3znd"}]}
        {"versionId": "3-QmVUpHdsP2LtPbuCVAmSApSDNfn9AeY3GVWuC9FXWByA3C", "versionTime": "2024-10-01T22:13:49Z", "parameters": {}, "state": {"@context": ["https://www.w3.org/ns/did/v1", "https://w3id.org/security/multikey/v1", "https://identity.foundation/.well-known/did-configuration/v1", "https://identity.foundation/linked-vp/contexts/v1"], "id": "did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example", "authentication": ["did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example#z6MkijyunEqPi7hzgJirb4tQLjztCPbJeeZvXEySuzbY6MLv"], "assertionMethod": ["did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example#z6MkijyunEqPi7hzgJirb4tQLjztCPbJeeZvXEySuzbY6MLv"], "verificationMethod": [{"id": "did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example#z6MkijyunEqPi7hzgJirb4tQLjztCPbJeeZvXEySuzbY6MLv", "controller": "did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example", "type": "Multikey", "publicKeyMultibase": "z6MkijyunEqPi7hzgJirb4tQLjztCPbJeeZvXEySuzbY6MLv"}], "service": [{"id": "did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example#domain", "type": "LinkedDomains", "serviceEndpoint": "https://domain.example"}, {"id": "did:webvh:QmaJp6pmb6RUk4oaDyWQcjeqYbvxsc3kvmHWPpz7B5JwDU:domain.example#whois", "type": "LinkedVerifiablePresentation", "serviceEndpoint": "https://domain.example/.well-known/whois.vc"}]}, "proof": [{"type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "verificationMethod": "did:key:z6MkoSFjacZb7R82htx8n1AkpgLQWR7CA6rigsc2VH9acLuF#z6MkoSFjacZb7R82htx8n1AkpgLQWR7CA6rigsc2VH9acLuF", "created": "2024-10-01T22:13:49Z", "proofPurpose": "authentication", "proofValue": "z32PcoCy9cRWBTUX8M9k5zNGunMnnn36B7yjwSnHJED7UfRC1EYJEDWiWP5yTdxy8QNKZRCitSDk4wzBtQM4nxNUj"}]}"#;
    #[test]
    fn test_did_log_entry_deserialization() {
        let entry: DidLogEntry = serde_json::from_str(JSON).unwrap();
        assert_eq!(
            entry.version_id,
            "1-QmRD52wqs942kZ2gs7UU9QmaopvqnMziqB4qgFDYsapCT9"
        );
        assert_eq!(entry.version_time, "2024-10-01T22:13:49Z");
        assert_eq!(entry.parameters.method.unwrap(), "did:webvh:0.5");
    }

    #[test]
    fn test_did_log_entries_deserialization() {
        let entries: Vec<DidLogEntry> = JSONL
            .lines()
            .map(|line| match serde_json::from_str(line) {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    panic!("Error: {:?}", e);
                }
            })
            .collect();

        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0].version_id,
            "1-QmRD52wqs942kZ2gs7UU9QmaopvqnMziqB4qgFDYsapCT9"
        );
        assert_eq!(
            entries[1].version_id,
            "2-QmV9Kh7GTCWBhxeKoZfWGC1QpJh1oQNhkf34RjpDZjsRhu"
        );
        assert_eq!(
            entries[2].version_id,
            "3-QmVUpHdsP2LtPbuCVAmSApSDNfn9AeY3GVWuC9FXWByA3C"
        );
    }

    #[test]
    fn test_log_entry_properties() {
        let entry: DidLogEntry = serde_json::from_str(JSON).unwrap();

        let (version_number, hash) = entry.parse_verion_id().unwrap();
        assert_eq!(version_number, 1);
        assert_eq!(hash, "QmRD52wqs942kZ2gs7UU9QmaopvqnMziqB4qgFDYsapCT9");

        let entry: DidLogEntry = serde_json::from_str(JSON).unwrap();
        let new_entry = entry.replace_to_scid_placeholder().unwrap();
        assert_eq!(new_entry.parameters.scid.unwrap(), "{SCID}");
        assert_eq!(new_entry.version_id, "{SCID}");
    }

    #[test]
    fn test_calc_entry_hash() {
        const JSON_LOG: &str = r#"{
  "versionId": "{SCID}",
  "versionTime": "2025-01-24T02:21:51Z",
  "parameters": {
    "updateKeys": [
      "z6MkjUuC31SMY2fengpaaDnQ9gFpjaWy4yMyfedAQYp1eSJZ"
    ],
    "method": "did:webvh:0.4",
    "scid": "{SCID}"
  },
  "state": {
    "@context": [
      "https://www.w3.org/ns/did/v1"
    ],
    "id": "did:webvh:{SCID}:example.com:eve"
  }
}"#;
        let mut entry: DidLogEntry = serde_json::from_str(JSON_LOG).unwrap();
        println!("ENTRY: {:?}\n", entry);
        let scid = entry.calc_entry_hash().unwrap();
        assert_eq!(scid, "QmbUzhqS4Fx6ueq6gopKQBNe2Dyj4dddCTyPuN4pncYxYG");

        entry.version_id = scid.to_string();
        entry.parameters.scid = Some(scid.to_string());
        let identifier = entry.state.id.get_method_specific_id();
        let identifier = identifier.replace("{SCID}", scid.as_str());
        entry.state.id = Did::new("webvh", identifier.as_str()).unwrap();
        println!("ENTRY_REPLACE_SCID: {:?}\n", entry);
        let entry_hash = entry.calc_entry_hash().unwrap();
        assert_eq!(entry_hash, "QmeyX9Tripap4bpri4324AUDCeUpBXKHRBHW89rnWa4mKw");
    }
}
