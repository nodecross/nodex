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
    let to_did = to_doc.clone().id;
    let message = Message::new().from(from_did).to(&[&to_did]).body(body)?;

    let public_key: x25519_dalek::PublicKey = to_doc.get_key("#encryptionKey")?.try_into()?;
    let public_key = public_key.as_bytes().to_vec();
    let public_key = Some(public_key);

    let seal_message = message
        .as_jwe(&CryptoAlgorithm::XC20P, public_key.clone())
        .kid(&to_did.into_inner())
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
    use super::{decrypt_message, encrypt_message, DidCommDecryptMessageError};
    use crate::did_webvh::domain::did::Did;
    use crate::did_webvh::domain::did_document::DidDocument;
    use crate::keyring::keypair::KeyPairing;
    use rand_core::OsRng;
    use serde_json::json;

    fn did_document(id: &str) -> (KeyPairing, DidDocument) {
        let from_keyring = KeyPairing::create_keyring(OsRng);
        let did = Did::new("web", id).unwrap();
        let vms = from_keyring.to_verification_methods(&did).unwrap();
        let mut doc = DidDocument::new(did);
        doc.verification_method = Some(vms);
        (from_keyring, doc)
    }

    #[rstest::fixture]
    fn from() -> (KeyPairing, DidDocument) {
        did_document("from")
    }

    #[rstest::fixture]
    fn to() -> (KeyPairing, DidDocument) {
        did_document("to")
    }

    #[rstest::fixture]
    fn other() -> (KeyPairing, DidDocument) {
        did_document("other")
    }

    #[rstest::rstest]
    fn test_generate_and_verify(from: (KeyPairing, DidDocument), to: (KeyPairing, DidDocument)) {
        let (from_key, from_doc) = from;
        let (to_key, to_doc) = to;
        let from_did = from_doc.id.clone();
        let message = json!({"test": "0123456789abcdef"});

        let res = encrypt_message(&message.to_string(), &from_did, &from_key, &to_doc).unwrap();

        let from_did_in_msg = res.find_sender().unwrap();
        assert_eq!(from_did_in_msg, from_did);

        let to_did_in_msg = res.find_receivers();
        assert_eq!(to_did_in_msg, vec![to_doc.id.clone().into_inner()]);

        let verified = decrypt_message(&res, &from_doc, &to_key).unwrap();
        assert_eq!(verified, message.to_string());
    }

    #[rstest::rstest]
    fn test_cannot_steal_message(
        from: (KeyPairing, DidDocument),
        to: (KeyPairing, DidDocument),
        other: (KeyPairing, DidDocument),
    ) {
        let (from_key, from_doc) = from;
        let (_, to_doc) = to;
        let (other_key, other_doc) = other;
        let from_did = from_doc.id.clone();

        let message = json!({"test": "0123456789abcdef"});

        let res = encrypt_message(&message.to_string(), &from_did, &from_key, &to_doc).unwrap();
        let verified = decrypt_message(&res, &other_doc, &other_key);

        assert!(matches!(
            verified,
            Err(DidCommDecryptMessageError::DecryptionFailed(_))
        ));
    }
}
