use super::did::Did;
use crate::keyring::jwk::Jwk;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: Did,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deactivated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_method: Option<Vec<VerificationMethod>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertion_method: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_agreement: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_invocation: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_delegation: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Vec<Service>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<Vec<Did>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub controller: Did,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_jwk: Option<Jwk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchain_account_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub service_endpoint: String,
}

impl DidDocument {
    pub fn new(id: Did) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/did/v1".to_string()],
            id,
            verification_method: None,
            authentication: None,
            assertion_method: None,
            key_agreement: None,
            capability_invocation: None,
            capability_delegation: None,
            service: None,
            also_known_as: None,
            controller: None,
            created: None,
            deactivated: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str =
        r#"{"@context":["https://www.w3.org/ns/did/v1"],"id":"did:web:example.com"}"#;

    #[test]
    fn test_did_document_new() {
        let did = Did::new("web", "example.com").unwrap();
        let did_document = DidDocument::new(did);
        assert_eq!(did_document.id.to_string(), "did:web:example.com");

        let json = serde_json::to_string(&did_document).unwrap();
        // remove all '\n' characters
        let json = json.chars().filter(|c| *c != '\n').collect::<String>();
        assert_eq!(json, JSON);
    }

    #[test]
    fn test_did_document_serde() {
        let did_document: DidDocument = serde_json::from_str(JSON).unwrap();
        assert_eq!(did_document.id.to_string(), "did:web:example.com");
    }

    #[test]
    fn test_did_document_serde_roundtrip() {
        let did_document: DidDocument = serde_json::from_str(JSON).unwrap();
        let json = serde_json::to_string(&did_document).unwrap();
        // remove all whitespace
        let json = json
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        // remove all '\n' characters
        let json = json.chars().filter(|c| *c != '\n').collect::<String>();
        assert_eq!(json, JSON);
    }
}
