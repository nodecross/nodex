use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::nodex::{keyring::secp256k1::Secp256k1, schema::general::GeneralVcDataModel, utils};

use super::jws::Jws;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
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
    pub did: Option<String>,
    pub key_id: Option<String>,
    pub context: Secp256k1,
}

#[derive(Debug, Error)]
pub enum CredentialSignerError {
    #[error("jws error: {0:?}")]
    JwsError(#[from] super::jws::JwsError),
    #[error("json parse error: {0:?}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("did is none. please set did")]
    DidIsNone,
    #[error("key_id is none. please set key_id")]
    KeyIdIsNone,
    #[error("proof not found")]
    ProofNotFound,
}

pub struct CredentialSigner {}

impl CredentialSigner {
    #[allow(dead_code)]
    const PROOF_KEY: &'static str = "proof";

    pub fn sign(
        object: &GeneralVcDataModel,
        suite: &CredentialSignerSuite,
    ) -> Result<GeneralVcDataModel, CredentialSignerError> {
        // FIXME:
        // if (Object.keys(object).indexOf(this.PROOF_KEY) !== -1) {
        //     throw new Error()
        // }

        let created = Utc::now().to_rfc3339();
        let jws = Jws::encode(&json!(object), &suite.context)?;

        let did = match &suite.did {
            Some(v) => v,
            None => return Err(CredentialSignerError::DidIsNone),
        };
        let key_id = match &suite.key_id {
            Some(v) => v,
            None => return Err(CredentialSignerError::KeyIdIsNone),
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
        let mut signed_object = json!(object);

        utils::json::merge(&mut signed_object, json!(proof));

        Ok(serde_json::from_value::<GeneralVcDataModel>(signed_object)?)
    }

    pub fn verify(
        mut object: GeneralVcDataModel,
        suite: &CredentialSignerSuite,
    ) -> Result<(GeneralVcDataModel, bool), CredentialSignerError> {
        // FIXME:
        // if (Object.keys(object).indexOf(this.PROOF_KEY) === -1) {
        //     throw new Error()
        // }

        let proof = object
            .proof
            .take()
            .ok_or(CredentialSignerError::ProofNotFound)?;

        // FIXME:
        // if (proof === undefined) {
        //     throw new NodeXNotCompatibleError()
        // }

        // FIXME:
        // const vm = utils.splitDid(proof.verificationMethod)
        // if (vm.keyId !== suite.keyId) {
        //     throw new NodeXNotCompatibleError()
        // }

        let jws = proof.jws;
        let payload = serde_json::to_value(&object)?;

        // NOTE: verify
        let verified = Jws::verify(&payload, &jws, &suite.context)?;

        Ok((object, verified))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::nodex::{
        keyring::{self, secp256k1::Secp256k1Context},
        schema::general::{CredentialSubject, Issuer},
    };

    use super::*;
    use rstest::*;

    #[fixture]
    fn secret_key() -> Vec<u8> {
        vec![
            0xc7, 0x39, 0x80, 0x5a, 0xb0, 0x3d, 0xa6, 0x2d, 0xdb, 0xe0, 0x33, 0x90, 0xac, 0xdf,
            0x76, 0x15, 0x64, 0x0a, 0xa6, 0xed, 0x31, 0xb8, 0xf1, 0x82, 0x43, 0xf0, 0x4a, 0x57,
            0x2c, 0x52, 0x8e, 0xdb,
        ]
    }

    #[fixture]
    fn public_key() -> Vec<u8> {
        vec![
            0x02, 0x70, 0x96, 0x45, 0x32, 0xf0, 0x83, 0xf4, 0x5f, 0xe8, 0xe8, 0xcc, 0xea, 0x96,
            0xa2, 0x2f, 0x60, 0x18, 0xd4, 0x6a, 0x40, 0x6f, 0x58, 0x3a, 0xb2, 0x26, 0xb1, 0x92,
            0x83, 0xaa, 0x60, 0x5c, 0x44,
        ]
    }

    // #[fixture]
    // fn message() -> String {
    //     String::from(r#"{"k":"0123456789abcdef"}"#)
    // }

    #[test]
    pub fn test_sign() {
        let context = match keyring::secp256k1::Secp256k1::new(&Secp256k1Context {
            public: public_key(),
            secret: secret_key(),
        }) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let model = GeneralVcDataModel {
            id: None,
            r#type: vec!["type".to_string()],
            issuer: Issuer {
                id: "issuer".to_string(),
            },
            context: vec!["context".to_string()],
            issuance_date: "issuance_date".to_string(),
            credential_subject: CredentialSubject {
                id: None,
                container: json!(r#"{"k":"0123456789abcdef"}"#),
            },
            expiration_date: None,
            proof: None,
        };

        let result = match CredentialSigner::sign(
            &model,
            &CredentialSignerSuite {
                did: Some("did:nodex:test:000000000000000000000000000000".to_string()),
                key_id: Some("signingKey".to_string()),
                context,
            },
        ) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        match result.proof {
            Some(proof) => {
                assert_eq!(proof.jws, "eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..Qc-NyzQu2v735_qPR72j1oqUDK1Ne4XQ7Lc66_x9tlMSeI9xmrgguEA8UmQyTM0cd13xkvpK4g-NEWJBp8_d_w");
                assert_eq!(proof.proof_purpose, "authentication");
                assert_eq!(proof.r#type, "EcdsaSecp256k1Signature2019");
                assert_eq!(
                    proof.verification_method,
                    "did:nodex:test:000000000000000000000000000000#signingKey"
                );
            }
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
            Err(_) => panic!(),
        };

        let model = GeneralVcDataModel {
            id: None,
            r#type: vec!["type".to_string()],
            issuer: Issuer {
                id: "issuer".to_string(),
            },
            context: vec!["context".to_string()],
            issuance_date: "issuance_date".to_string(),
            credential_subject: CredentialSubject {
                id: None,
                container: json!(r#"{"k":"0123456789abcdef"}"#),
            },
            expiration_date: None,
            proof: None,
        };

        let vc = match CredentialSigner::sign(
            &model,
            &CredentialSignerSuite {
                did: Some("did:nodex:test:000000000000000000000000000000".to_string()),
                key_id: Some("signingKey".to_string()),
                context: context.clone(),
            },
        ) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let (verified_model, verified) = match CredentialSigner::verify(
            vc,
            &CredentialSignerSuite {
                did: None,
                key_id: None,
                context,
            },
        ) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(verified);
        assert_eq!(model, verified_model);
    }
}
