use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
  #[serde(rename = "@context")]
  pub context: String,
  pub did_document: DidDocument,
  pub method_metadata: MethodMetadata,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
  pub id: String,
  #[serde(rename = "@context")]
  pub context: (String, Context),
  pub service: Vec<::serde_json::Value>,
  pub public_key: Vec<DidPublicKey>,
  pub authentication: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
  #[serde(rename = "@base")]
  pub base: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidPublicKey {
  pub id: String,
  pub controller: String,
  #[serde(rename = "type")]
  pub type_field: String,
  pub public_key_jwk: KeyPairSecp256K1,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyPairSecp256K1 {
  pub kty: String,
  pub crv: String,
  pub x: String,
  pub y: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodMetadata {
  pub published: bool,
  pub recovery_commitment: String,
  pub update_commitment: String,
}
