use crate::did_webvh::domain::did::Did;
use crate::did_webvh::domain::did_document::{DidDocument, GetPublicKeyError};
use crate::didcomm::types::DidCommMessage;
use crate::keyring::jwk::{JwkToK256Error, JwkToX25519Error};
use crate::keyring::keypair::{KeyPair, KeyPairing};
pub use didcomm_rs;
use didcomm_rs::crypto::SignatureAlgorithm;
use didcomm_rs::{crypto::CryptoAlgorithm, Message};
use k256::elliptic_curve::sec1::ToEncodedPoint;
pub use serde_json;
use thiserror::Error;

pub fn encrypt_message(
    body: &str,
    from_did: &Did,
    from_keyring: &KeyPairing,
    to_doc: &DidDocument,
) -> Result<DidCommMessage, DidCommEncryptMessageError> {
    let to_did = &to_doc.id;
    let message = Message::new().from(from_did).to(&[to_did]).body(&body)?;

    let public_key: x25519_dalek::PublicKey = to_doc.get_key("#encryptionKey")?.try_into()?;
    let public_key = public_key.as_bytes().to_vec();
    let public_key = Some(public_key);

    let seal_message = message
        .as_jwe(&CryptoAlgorithm::XC20P, public_key.clone())
        .seal_signed(
            from_keyring.encrypt.get_secret_key().as_bytes(),
            Some(vec![public_key]),
            SignatureAlgorithm::Es256k,
            &from_keyring.sign.get_secret_key().to_bytes(),
        )?;

    Ok(serde_json::from_str::<DidCommMessage>(&seal_message)?)
}

pub fn decrypt_message(
    message: &DidCommMessage,
    from_doc: &DidDocument,
    to_keyring: &KeyPairing,
) -> Result<String, DidCommDecryptMessageError> {
    let encryption_public_key: x25519_dalek::PublicKey =
        from_doc.get_key("#encryptionKey")?.try_into()?;
    let encryption_public_key = encryption_public_key.as_bytes().to_vec();
    let signing_public_key: k256::PublicKey = from_doc.get_key("#signingKey")?.try_into()?;
    let signing_public_key = signing_public_key.to_encoded_point(false);

    let message = Message::receive(
        &serde_json::to_string(&message)?,
        Some(to_keyring.encrypt.get_secret_key().as_bytes().as_ref()),
        Some(encryption_public_key),
        Some(signing_public_key.as_bytes()),
    )?;

    Ok(message.get_body()?)
}

#[derive(Debug, Error)]
pub enum DidCommEncryptMessageError {
    #[error(transparent)]
    JwkToX25519(#[from] JwkToX25519Error),
    #[error("failed to get did document: {0}")]
    DidDocNotFound(String),
    #[error("did public key not found. did: {0}")]
    DidPublicKeyNotFound(#[from] GetPublicKeyError),
    #[error("failed to encrypt message with error: {0}")]
    EncryptionFailed(#[from] didcomm_rs::Error),
    #[error("failed serialize/deserialize: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum DidCommDecryptMessageError {
    #[error(transparent)]
    JwkToX25519(#[from] JwkToX25519Error),
    #[error(transparent)]
    JwkToK256(#[from] JwkToK256Error),
    #[error("failed to get did document: {0}")]
    DidDocNotFound(String),
    #[error("did public key not found. did: {0}")]
    DidPublicKeyNotFound(#[from] GetPublicKeyError),
    #[error("failed to decrypt message: {0:?}")]
    DecryptionFailed(#[from] didcomm_rs::Error),
    #[error("failed serialize/deserialize: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, iter::FromIterator as _};

    use chrono::{DateTime, Utc};
    use rand_core::OsRng;
    use serde_json::{json, Value};

    // use super::*;
    use super::DidCommEncryptedService;
    use crate::{
        did::did_repository::{mocks::MockDidRepository, GetPublicKeyError},
        didcomm::{
            encrypted::{DidCommEncryptedServiceGenerateError, DidCommEncryptedServiceVerifyError},
            test_utils::create_random_did,
            types::DidCommMessage,
        },
        keyring::keypair::KeyPairing,
        verifiable_credentials::types::VerifiableCredentials,
    };

    #[tokio::test]
    async fn test_generate_and_verify() {
        let from_did = create_random_did();
        let to_did = create_random_did();

        let to_keyring = KeyPairing::create_keyring(OsRng);
        let from_keyring = KeyPairing::create_keyring(OsRng);

        let repo = MockDidRepository::from_single(BTreeMap::from_iter([
            (from_did.clone(), from_keyring.clone()),
            (to_did.clone(), to_keyring.clone()),
        ]));

        let message = json!({"test": "0123456789abcdef"});
        let issuance_date = Utc::now();

        let model = VerifiableCredentials::new(from_did.clone(), message.clone(), issuance_date);
        let res = repo
            .generate(model, &from_keyring, &to_did, None)
            .await
            .unwrap();

        let verified = repo.verify(&to_keyring, &res).await.unwrap();
        let verified = verified.message;

        assert_eq!(verified.issuer.id, from_did);
        assert_eq!(verified.credential_subject.container, message);
    }

    mod generate_failed {
        use super::*;
        use crate::did::did_repository::mocks::NoPublicKeyDidRepository;

        #[tokio::test]
        async fn test_did_not_found() {
            let from_did = create_random_did();
            let to_did = create_random_did();

            let from_keyring = KeyPairing::create_keyring(OsRng);

            let repo = MockDidRepository::from_single(BTreeMap::from_iter([(
                from_did.clone(),
                from_keyring.clone(),
            )]));

            let message = json!({"test": "0123456789abcdef"});
            let issuance_date = Utc::now();

            let model = VerifiableCredentials::new(from_did, message, issuance_date);
            let res = repo
                .generate(model, &from_keyring, &to_did, None)
                .await
                .unwrap_err();

            if let DidCommEncryptedServiceGenerateError::DidDocNotFound(did) = res {
                assert_eq!(did, to_did);
            } else {
                panic!("unexpected result: {:?}", res);
            }
        }

        #[tokio::test]
        async fn test_did_public_key_not_found() {
            let from_did = create_random_did();
            let to_did = create_random_did();

            let from_keyring = KeyPairing::create_keyring(OsRng);

            let repo = NoPublicKeyDidRepository;

            let message = json!({"test": "0123456789abcdef"});
            let issuance_date = Utc::now();

            let model = VerifiableCredentials::new(from_did, message, issuance_date);
            let res = repo
                .generate(model, &from_keyring, &to_did, None)
                .await
                .unwrap_err();

            if let DidCommEncryptedServiceGenerateError::DidPublicKeyNotFound(
                GetPublicKeyError::PublicKeyNotFound(did),
            ) = res
            {
                assert_eq!(did, to_did);
            } else {
                panic!("unexpected result: {:?}", res);
            }
        }
    }

    mod verify_failed {
        use super::*;
        use crate::did::did_repository::mocks::NoPublicKeyDidRepository;

        async fn create_didcomm(
            from_did: &str,
            to_did: &str,
            from_keyring: &KeyPairing,
            to_keyring: &KeyPairing,
            message: &Value,
            metadata: Option<&Value>,
            issuance_date: DateTime<Utc>,
        ) -> DidCommMessage {
            let repo = MockDidRepository::from_single(BTreeMap::from_iter([(
                to_did.to_string(),
                to_keyring.clone(),
            )]));

            let model =
                VerifiableCredentials::new(from_did.to_string(), message.clone(), issuance_date);

            repo.generate(model, from_keyring, to_did, metadata)
                .await
                .unwrap()
        }

        #[tokio::test]
        async fn test_did_not_found() {
            let from_did = create_random_did();
            let to_did = create_random_did();

            let to_keyring = KeyPairing::create_keyring(OsRng);
            let from_keyring = KeyPairing::create_keyring(OsRng);

            let message = json!({"test": "0123456789abcdef"});
            let issuance_date = Utc::now();

            let res = create_didcomm(
                &from_did,
                &to_did,
                &from_keyring,
                &to_keyring,
                &message,
                None,
                issuance_date,
            )
            .await;

            let repo = MockDidRepository::from_single(BTreeMap::from_iter([(
                to_did.clone(),
                to_keyring.clone(),
            )]));
            let res = repo.verify(&from_keyring, &res).await.unwrap_err();

            if let DidCommEncryptedServiceVerifyError::DidDocNotFound(did) = res {
                assert_eq!(did, from_did);
            } else {
                panic!("unexpected result: {:?}", res);
            }
        }

        #[tokio::test]
        async fn test_cannot_steal_message() {
            let from_did = create_random_did();
            let to_did = create_random_did();
            let other_did = create_random_did();

            let to_keyring = KeyPairing::create_keyring(OsRng);
            let from_keyring = KeyPairing::create_keyring(OsRng);
            let other_keyring = KeyPairing::create_keyring(OsRng);

            let message = json!({"test": "0123456789abcdef"});
            let issuance_date = Utc::now();

            let res = create_didcomm(
                &from_did,
                &to_did,
                &from_keyring,
                &to_keyring,
                &message,
                None,
                issuance_date,
            )
            .await;

            let repo = MockDidRepository::from_single(BTreeMap::from_iter([
                (from_did.clone(), from_keyring.clone()),
                (to_did.clone(), to_keyring.clone()),
                (other_did.clone(), other_keyring.clone()),
            ]));

            let res = repo.verify(&other_keyring, &res).await.unwrap_err();

            if let DidCommEncryptedServiceVerifyError::DecryptFailed(_) = res {
            } else {
                panic!("unexpected result: {:?}", res);
            }
        }

        #[tokio::test]
        async fn test_did_public_key_not_found() {
            let from_did = create_random_did();
            let to_did = create_random_did();

            let to_keyring = KeyPairing::create_keyring(OsRng);
            let from_keyring = KeyPairing::create_keyring(OsRng);

            let message = json!({"test": "0123456789abcdef"});
            let issuance_date = Utc::now();

            let res = create_didcomm(
                &from_did,
                &to_did,
                &from_keyring,
                &to_keyring,
                &message,
                None,
                issuance_date,
            )
            .await;

            let repo = NoPublicKeyDidRepository;

            let res = repo.verify(&from_keyring, &res).await.unwrap_err();

            if let DidCommEncryptedServiceVerifyError::DidPublicKeyNotFound(
                GetPublicKeyError::PublicKeyNotFound(did),
            ) = res
            {
                assert_eq!(did, from_did);
            } else {
                panic!("unexpected result: {:?}", res);
            }
        }
    }
}
