use super::did_document::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DIDCreateRequest {
    pub public_keys: Vec<PublicKeyPayload>,
    pub commitment_keys: CommitmentKeys,
    pub service_endpoints: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitmentKeys {
    pub recovery: KeyPairSecp256K1,
    pub update: KeyPairSecp256K1,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub jwk: KeyPairSecp256K1,
    pub purpose: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DIDCreatePayload {
    #[serde(rename = "type")]
    pub type_field: String,
    pub delta: String,
    pub suffix_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DIDReplacePayload {
    pub public_keys: Vec<PublicKeyPayload>,
    pub service_endpoints: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DIDReplaceAction {
    pub action: String,
    pub document: DIDReplacePayload,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DIDReplaceDeltaObject {
    pub patches: Vec<DIDReplaceAction>,
    pub update_commitment: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DIDReplaceSuffixObject {
    pub delta_hash: String,
    pub recovery_commitment: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DIDResolutionRequest {
    pub did: String,
}

pub type KV = HashMap<String, String>;