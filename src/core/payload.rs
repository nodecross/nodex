use super::super::utils::{encoder::*, multihasher::*};
use super::interfaces::did_operation::*;
pub struct Payload {}

impl Payload {
  pub fn did_create_payload(params: DIDCreateRequest) -> DIDCreatePayload {
    let document: DIDReplacePayload = DIDReplacePayload {
      public_keys: params.public_keys,
      service_endpoints: Vec::new(),
    };

    let patch: DIDReplaceAction = DIDReplaceAction {
      action: "replace".to_string(),
      document,
    };

    let delta: DIDReplaceDeltaObject = DIDReplaceDeltaObject {
      patches: vec![patch],
      update_commitment: Multihasher::canonicalize_then_double_hash_then_encode(
        serde_json::to_value(&params.commitment_keys.update).unwrap(),
      ),
    };

    let delta_serde: serde_json::Value = serde_json::to_value(&delta).unwrap();
    let delta_str: &str = &delta_serde.to_string();
    let delta_u8: &[u8] = delta_str.as_bytes();
    let delta_buf: Vec<u8> = delta_u8.to_vec();
    let delta_hash: String = Encoder::encode(Multihasher::hash(delta_buf.to_owned()));
    let suffix_data: DIDReplaceSuffixObject = DIDReplaceSuffixObject {
      delta_hash,
      recovery_commitment: Multihasher::canonicalize_then_double_hash_then_encode(
        serde_json::to_value(&params.commitment_keys.update).unwrap(),
      ),
    };
    let delta_encoded_string: String = Encoder::encode(delta_buf);
    let suffix_serde: serde_json::Value = serde_json::to_value(&suffix_data).unwrap();
    let suffix_str: &str = &suffix_serde.to_string();
    let suffix_u8: &[u8] = suffix_str.as_bytes();
    let suffix_buf: Vec<u8> = suffix_u8.to_vec();
    let suffix_data_encoded_string: String = Encoder::encode(suffix_buf);

    let payload: DIDCreatePayload = DIDCreatePayload {
      type_field: "create".to_string(),
      delta: delta_encoded_string,
      suffix_data: suffix_data_encoded_string,
    };

    payload
  }
}
