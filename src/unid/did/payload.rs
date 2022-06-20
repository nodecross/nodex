use serde::{Deserialize, Serialize};

use crate::unid::utils::algorithms::base64_url;
use crate::unid::utils::algorithms::multihash;
use crate::unid::errors::UNiDError;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyPairSecp256K1 {
    pub r#kty: String,
    pub r#crv: String,
    pub r#x: String,
    pub r#y: String,
    pub r#d: Option<String>,
    pub r#kid: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicKeyPayload {
    pub r#id  : String,
    pub r#type: String,
    pub r#jwk : KeyPairSecp256K1,
    pub r#purpose: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplacePayload {
    pub r#public_keys: Vec<PublicKeyPayload>,
    pub r#service_endpoints: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplaceAction {
    pub r#action: String,
    pub r#document: ReplacePayload,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplaceDeltaObject {
    pub patches: Vec<ReplaceAction>,
    pub update_commitment: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplaceSuffixObject {
    pub delta_hash: String,
    pub recovery_commitment: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub r#type : String,
    pub r#delta: String,
    pub r#suffix_data: String,
}

impl Payload {
    pub fn new(public_keys: &[PublicKeyPayload], recovery_key: &KeyPairSecp256K1, update_key: &KeyPairSecp256K1) -> Result<Payload, UNiDError> {
        let document = ReplacePayload {
            public_keys: public_keys.to_vec(),
            service_endpoints: Vec::from([])
        };
        let patch = ReplaceAction {
            action: "replace".to_string(),
            document,
        };

        let delta = ReplaceDeltaObject {
            patches: [patch].to_vec(),
            update_commitment: multihash::Multihash::canonicalize_then_double_hash_then_encode(&serde_json::to_string(&update_key).unwrap().as_bytes().to_vec())
        };
        let delta_buffer = match serde_json::to_string(&delta) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let delta_hash = base64_url::Base64Url::encode(
            &multihash::Multihash::hash(&delta_buffer.as_bytes().to_vec())
        );

        let suffix_data = ReplaceSuffixObject {
            delta_hash,
            recovery_commitment: multihash::Multihash::canonicalize_then_double_hash_then_encode(&serde_json::to_string(&recovery_key).unwrap().as_bytes().to_vec())
        };
        let delta_encoded_string = base64_url::Base64Url::encode(&delta_buffer.as_bytes().to_vec());
        let suffix_data_encoded_string = base64_url::Base64Url::encode(&serde_json::to_string(&suffix_data).unwrap().into_bytes());

        Ok(Payload {
            r#type: "create".to_string(),
            delta: delta_encoded_string,
            suffix_data: suffix_data_encoded_string,
        })
    }

    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use serde_json::json;

    use crate::unid::utils::random;

    #[fixture]
    fn message() -> String {
        String::from(r#"{"k":"UNiD"}"#)
    }

    #[test]
    fn test_new() {
        let private_key = random::Random::bytes(&32).unwrap();

        let x = &private_key[0..16];
        let y = &private_key[16..];

        let k = KeyPairSecp256K1 {
            kty: "EC".to_string(),
            crv: "secp256k1".to_string(),
            kid: None,
            d: None,
            x: base64_url::Base64Url::encode(&x.to_vec()),
            y: base64_url::Base64Url::encode(&y.to_vec()),
        };

        let public_key = PublicKeyPayload {
            r#id: "".to_string(),
            r#type: "".to_string(),
            r#jwk: k.clone(),
            purpose: ["".to_string()].to_vec()
        };

        let result = match Payload::new(
            &[public_key].to_vec(),
            &k,
            &k,
        ) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(json!(result)["type"], "create".to_string());
        assert_eq!(json!(result)["delta"], "eyJwYXRjaGVzIjpbeyJhY3Rpb24iOiJyZXBsYWNlIiwiZG9jdW1lbnQiOnsicHVibGljX2tleXMiOlt7ImlkIjoiIiwidHlwZSI6IiIsImp3ayI6eyJrdHkiOiJFQyIsImNydiI6InNlY3AyNTZrMSIsIngiOiJpaFNvUUtUZXQxWUN0ZE9GX0hCMlpBIiwieSI6IkpZdmZlNWVDVUtnWkpPbGNuRnUzYnciLCJkIjpudWxsLCJraWQiOm51bGx9LCJwdXJwb3NlIjpbIiJdfV0sInNlcnZpY2VfZW5kcG9pbnRzIjpbXX19XSwidXBkYXRlX2NvbW1pdG1lbnQiOiJFaURuNXhqZ19HSFR1NW93V1JQWVh5ZV91VGJUWlZyWUFILVg4eGlZVDhQTWFBIn0".to_string());
        assert_eq!(json!(result)["suffix_data"], "eyJkZWx0YV9oYXNoIjoiRWlBbTFmY1duWERzQTFkNTRzNE1DVlh6VzEtYjVNOXFMbUxESVdSVmVxOVNnQSIsInJlY292ZXJ5X2NvbW1pdG1lbnQiOiJFaURuNXhqZ19HSFR1NW93V1JQWVh5ZV91VGJUWlZyWUFILVg4eGlZVDhQTWFBIn0".to_string());
    }
}