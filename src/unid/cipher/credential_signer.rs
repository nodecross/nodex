use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};

use crate::{unid::{keyring::secp256k1::Secp256k1, errors::UNiDError, utils, schema::general::GeneralVcDataModel}, services::unid::UNiD};

use super::jws::JWS;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Proof {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "proofPurpose")]
    pub proof_purpose: String,

    #[serde(rename = "created")]
    pub created: String,

    #[serde(rename = "verificationMethod")]
    pub verification_method: String,

    #[serde(rename = "jws")]
    pub jws: String,

    #[serde(rename = "controller")]
    pub controller: Option<String>,

    #[serde(rename = "challenge")]
    pub challenge: Option<String>,

    #[serde(rename = "domain")]
    pub domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofContext {
    #[serde(rename = "proof")]
    pub proof: Option<Proof>,
}

pub struct CredentialSignerSuite {
    pub did    : Option<String>,
    pub key_id : Option<String>,
    pub context: Secp256k1,
}

pub struct CredentialSigner {}

impl CredentialSigner {
    const PROOF_KEY: &'static str = "proof";

    pub fn sign(object: &GeneralVcDataModel, suite: &CredentialSignerSuite) -> Result<GeneralVcDataModel, UNiDError> {
        // FIXME:
        // if (Object.keys(object).indexOf(this.PROOF_KEY) !== -1) {
        //     throw new Error()
        // }

        let created = Utc::now().to_rfc3339();
        let jws = match JWS::encode(&json!(object), &suite.context) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let did = match &suite.did {
            Some(v) => v,
            None => return Err(UNiDError{})
        };
        let key_id = match &suite.key_id {
            Some(v) => v,
            None => return Err(UNiDError{})
        };

        let proof: ProofContext = ProofContext {
            proof: Some(Proof {
                r#type: "EcdsaSecp256k1Signature2019".to_string(),
                proof_purpose: "authentication".to_string(),
                created,
                verification_method: format!("{}#{}", did, key_id),
                jws,
                domain: None,
                controller: None,
                challenge: None,
            }),
        };

        // NOTE: sign
        let mut signed_object = json!(object.clone());

        utils::json::merge(&mut signed_object, json!(proof));

        match serde_json::from_value::<GeneralVcDataModel>(signed_object) {
            Ok(v) => Ok(v),
            Err(_) => Err(UNiDError{})
        }
    }

    pub fn verify(object: &GeneralVcDataModel, suite: &CredentialSignerSuite) -> Result<(Value, bool), UNiDError> {
        // FIXME:
        // if (Object.keys(object).indexOf(this.PROOF_KEY) === -1) {
        //     throw new Error()
        // }

        let mut serialized = json!(&object);

        let proof = match serde_json::from_value::<Proof>(serialized["proof"].take()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        // FIXME:
        // if (proof === undefined) {
        //     throw new UNiDNotCompatibleError()
        // }

        // FIXME:
        // const vm = utils.splitDid(proof.verificationMethod)
        // if (vm.keyId !== suite.keyId) {
        //     throw new UNiDNotCompatibleError()
        // }

        let jws = proof.jws;
        let payload = match serde_json::from_value::<GeneralVcDataModel>(serialized) {
            Ok(v) => json!(v),
            Err(_) => return Err(UNiDError{})
        };

        // NOTE: verify
        let verified = match JWS::verify(&payload, &jws, &suite.context) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok((payload, verified))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::unid::{keyring::{self, secp256k1::Secp256k1Context}, schema::general::{CredentialSubject, Issuer}};

    use super::*;
    use rstest::*;

    #[fixture]
    fn secret_key() -> Vec<u8> {
        vec![
            0xc7, 0x39, 0x80, 0x5a, 0xb0, 0x3d, 0xa6, 0x2d, 0xdb, 0xe0,
            0x33, 0x90, 0xac, 0xdf, 0x76, 0x15, 0x64, 0x0a, 0xa6, 0xed,
            0x31, 0xb8, 0xf1, 0x82, 0x43, 0xf0, 0x4a, 0x57, 0x2c, 0x52,
            0x8e, 0xdb,
        ]
    }

    #[fixture]
    fn public_key() -> Vec<u8> {
        vec![
            0x02, 0x70, 0x96, 0x45, 0x32, 0xf0, 0x83, 0xf4, 0x5f, 0xe8,
            0xe8, 0xcc, 0xea, 0x96, 0xa2, 0x2f, 0x60, 0x18, 0xd4, 0x6a,
            0x40, 0x6f, 0x58, 0x3a, 0xb2, 0x26, 0xb1, 0x92, 0x83, 0xaa,
            0x60, 0x5c, 0x44,
        ]
    }

    #[fixture]
    fn message() -> String {
        String::from(r#"{"message":"Hello, UNiD!"}"#)
    }

    #[test]
    pub fn test_sign() {
        let context = match keyring::secp256k1::Secp256k1::new(&Secp256k1Context {
            public: public_key(),
            secret: secret_key(),
        }) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let model = GeneralVcDataModel {
            id: None,
            r#type: vec![ "type".to_string() ],
            issuer: Issuer { id: "issuer".to_string() },
            context: vec![ "context".to_string() ],
            issuance_date: "issuance_date".to_string(),
            credential_subject: CredentialSubject {
                id: None,
                container: json!(r#"{"key":"value"}"#)
            },
            expiration_date: None,
            proof: None,
        };

        let result = match CredentialSigner::sign(&model, &CredentialSignerSuite {
            did: Some("did:unid:test:000000000000000000000000000000".to_string()),
            key_id: Some("signingKey".to_string()),
            context,
        }) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        match result.proof {
            Some(proof) => {
                assert_eq!(proof.jws, "eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..sJDXJkbpBph11FAVZIxSE7lW3-6HjzM8edL0npwkgfRM1aIPpG7yqtN4AP_Q30Agkam-sk9Q1Lty2i-ZW1-4Iw");
                assert_eq!(proof.proof_purpose, "authentication");
                assert_eq!(proof.r#type, "EcdsaSecp256k1Signature2019");
                assert_eq!(proof.verification_method, "did:unid:test:000000000000000000000000000000#signingKey");
            },
            None => panic!(),
        }
    }

    #[test]
    pub fn test_verify() {
        let context = match keyring::secp256k1::Secp256k1::new(&Secp256k1Context {
            public: public_key(),
            secret: secret_key(),
        }) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let model = GeneralVcDataModel {
            id: None,
            r#type: vec![ "type".to_string() ],
            issuer: Issuer { id: "issuer".to_string() },
            context: vec![ "context".to_string() ],
            issuance_date: "issuance_date".to_string(),
            credential_subject: CredentialSubject {
                id: None,
                container: json!(r#"{"key":"value"}"#)
            },
            expiration_date: None,
            proof: None,
        };

        let vc = match CredentialSigner::sign(&model, &CredentialSignerSuite {
            did: Some("did:unid:test:000000000000000000000000000000".to_string()),
            key_id: Some("signingKey".to_string()),
            context: context.clone(),
        }) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let (result, verified) = match CredentialSigner::verify(&vc, &CredentialSignerSuite {
            did: None,
            key_id: None,
            context: context.clone(),
        }) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let verified_model = match serde_json::from_value::<GeneralVcDataModel>(result) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(verified, true);
        assert_eq!(model, verified_model);
    }
}