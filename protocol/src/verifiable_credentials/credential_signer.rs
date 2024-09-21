use serde_json::json;
use thiserror::Error;

use super::types::Proof;
use crate::{
    keyring::keypair::{K256KeyPair, KeyPair},
    verifiable_credentials::{jws, types::VerifiableCredentials},
};

pub struct CredentialSignerSuite<'a> {
    pub did: &'a str,
    pub key_id: &'a str,
    pub context: &'a K256KeyPair,
}

#[derive(Debug, Error)]
pub enum CredentialSignerSignError {
    #[error("jws error: {0:?}")]
    Jws(#[from] jws::JwsEncodeError),
    #[error("json parse error: {0:?}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum CredentialSignerVerifyError {
    #[error("jws error: {0:?}")]
    Jws(#[from] jws::JwsDecodeError),
    #[error("json parse error: {0:?}")]
    Json(#[from] serde_json::Error),
    #[error("proof not found")]
    ProofNotFound,
}

pub struct CredentialSigner {}

impl CredentialSigner {
    pub fn sign(
        mut object: VerifiableCredentials,
        suite: CredentialSignerSuite,
    ) -> Result<VerifiableCredentials, CredentialSignerSignError> {
        let jws = jws::sign(&json!(object), &suite.context.get_secret_key())?;
        let did = suite.did;
        let key_id = suite.key_id;
        object.proof = Some(Proof {
            r#type: "EcdsaSecp256k1Signature2019".to_string(),
            proof_purpose: "authentication".to_string(),
            // Assume that object.issuance_date is correct data
            created: object.issuance_date,
            verification_method: format!("{}#{}", did, key_id),
            jws,
            domain: None,
            controller: None,
            challenge: None,
        });
        Ok(object)
    }

    pub fn verify(
        mut object: VerifiableCredentials,
        public_key: &k256::PublicKey,
    ) -> Result<VerifiableCredentials, CredentialSignerVerifyError> {
        let proof = object.proof.take().ok_or(CredentialSignerVerifyError::ProofNotFound)?;
        let jws = proof.jws;
        let payload = serde_json::to_value(&object)?;
        jws::verify(&payload, &jws, public_key)?;
        Ok(object)
    }
}

#[cfg(test)]
pub mod tests {

    use chrono::{DateTime, Utc};

    use super::*;
    use crate::verifiable_credentials::types::{CredentialSubject, Issuer};

    const PRIVATE_KEY: [u8; 32] = [
        0xc7, 0x39, 0x80, 0x5a, 0xb0, 0x3d, 0xa6, 0x2d, 0xdb, 0xe0, 0x33, 0x90, 0xac, 0xdf, 0x76,
        0x15, 0x64, 0x0a, 0xa6, 0xed, 0x31, 0xb8, 0xf1, 0x82, 0x43, 0xf0, 0x4a, 0x57, 0x2c, 0x52,
        0x8e, 0xdb,
    ];

    const PUBLIC_KEY: [u8; 33] = [
        0x02, 0x70, 0x96, 0x45, 0x32, 0xf0, 0x83, 0xf4, 0x5f, 0xe8, 0xe8, 0xcc, 0xea, 0x96, 0xa2,
        0x2f, 0x60, 0x18, 0xd4, 0x6a, 0x40, 0x6f, 0x58, 0x3a, 0xb2, 0x26, 0xb1, 0x92, 0x83, 0xaa,
        0x60, 0x5c, 0x44,
    ];

    #[test]
    pub fn test_public_key() {
        let sk = k256::SecretKey::from_slice(&PRIVATE_KEY).unwrap();
        let context = K256KeyPair::new(sk);
        assert_eq!(
            context.get_public_key(),
            k256::PublicKey::from_sec1_bytes(&PUBLIC_KEY).unwrap()
        );
    }

    #[test]
    pub fn test_sign() {
        let sk = k256::SecretKey::from_slice(&PRIVATE_KEY).unwrap();
        let context = K256KeyPair::new(sk);

        let model = VerifiableCredentials {
            id: None,
            r#type: vec!["type".to_string()],
            issuer: Issuer { id: "issuer".to_string() },
            context: vec!["context".to_string()],
            issuance_date: DateTime::parse_from_rfc3339("2024-07-19T06:06:51.361316372Z")
                .unwrap()
                .to_utc(),
            credential_subject: CredentialSubject {
                id: None,
                container: json!(r#"{"k":"0123456789abcdef"}"#),
            },
            expiration_date: None,
            proof: None,
        };

        let result = CredentialSigner::sign(
            model,
            CredentialSignerSuite {
                did: "did:nodex:test:000000000000000000000000000000",
                key_id: "signingKey",
                context: &context,
            },
        )
        .unwrap();

        match result.proof {
            Some(proof) => {
                assert_eq!(
                    proof.jws,
                    "eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..LK8OcOuMgWU4Y5Zpz9jeQ8b5UsgDmjKJTBpuxFepGlp-hGVHVgyZz8QkZseqQRdUXn6JouVYo1jFsQCq_7p7ig"
                );
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
        let sk = k256::SecretKey::from_slice(&PRIVATE_KEY).unwrap();
        let context = K256KeyPair::new(sk);

        let model = VerifiableCredentials {
            id: None,
            r#type: vec!["type".to_string()],
            issuer: Issuer { id: "issuer".to_string() },
            context: vec!["context".to_string()],
            issuance_date: Utc::now(),
            credential_subject: CredentialSubject {
                id: None,
                container: json!(r#"{"k":"0123456789abcdef"}"#),
            },
            expiration_date: None,
            proof: None,
        };

        let vc = CredentialSigner::sign(
            model.clone(),
            CredentialSignerSuite {
                did: "did:nodex:test:000000000000000000000000000000",
                key_id: "signingKey",
                context: &context,
            },
        )
        .unwrap();

        let verified_model = CredentialSigner::verify(vc, &context.get_public_key()).unwrap();

        assert_eq!(model, verified_model);
    }
}
