use crate::did_webvh::domain::did::Did;
use crate::did_webvh::domain::did_document::GetPublicKeyError;
use crate::did_webvh::service::resolver::resolver_service::DidWebvhResolverService;
use crate::keyring::jwk::JwkToEd25519Error;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use coset::{iana, CborSerializable, CoseError};
use ed25519_dalek::{Signature, SignatureError, Signer, SigningKey, Verifier, VerifyingKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use validator::Validate;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Token {
    pub did: Did,
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
}

impl Token {
    pub fn new(did: Did) -> Self {
        Self {
            did,
            exp: Utc::now() + std::time::Duration::from_secs(3600),
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct WithToken<T> {
    pub token: Token,
    pub inner: T,
}

#[derive(Debug, thiserror::Error)]
pub enum SignMessageError {
    #[error(transparent)]
    Cbor(ciborium::ser::Error<std::io::Error>),
    #[error(transparent)]
    Signature(SignatureError),
    #[error("cose error: {0}")]
    Cose(CoseError),
}

pub fn to_message<M: Serialize>(message: &M) -> Result<Vec<u8>, SignMessageError> {
    let mut c = Cursor::new(Vec::new());
    ciborium::into_writer(message, &mut c).map_err(SignMessageError::Cbor)?;
    Ok(c.into_inner())
}

pub fn sign_message<M: Serialize>(
    signing_key: &SigningKey,
    message: &WithToken<M>,
) -> Result<Vec<u8>, SignMessageError> {
    let message = to_message(message)?;
    let protected = coset::HeaderBuilder::new()
        .algorithm(iana::Algorithm::EdDSA)
        .build();
    let sign1 = coset::CoseSign1Builder::new()
        .protected(protected)
        .payload(message)
        .try_create_signature(b"", |pt| {
            signing_key.try_sign(pt).map(|s| s.to_bytes().into())
        })
        .map_err(SignMessageError::Signature)?
        .build();
    let sign1_data = sign1.to_vec().map_err(SignMessageError::Cose)?;
    Ok(sign1_data)
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeMessageError<F: std::error::Error + Send + Sync> {
    #[error(transparent)]
    Cbor(ciborium::de::Error<std::io::Error>),
    #[error("cose error: {0}")]
    Cose(CoseError),
    #[error("payload is empty")]
    PayloadEmpty,
    #[error("public key is empty")]
    NotFoundPubkey,
    #[error(transparent)]
    GetPubkey(#[from] GetPublicKeyError),
    #[error(transparent)]
    GetDidDocument(F),
    #[error("expired message")]
    Expired,
    #[error("incompatible array size: {0}")]
    VecToArray(usize),
    #[error(transparent)]
    Signature(SignatureError),
    #[error(transparent)]
    JwkToEd25519(#[from] JwkToEd25519Error),
}

#[trait_variant::make(Send)]
pub trait MessageVerifier: DidWebvhResolverService {
    async fn verify_message<M>(
        &mut self,
        data: &[u8],
    ) -> Result<WithToken<M>, DecodeMessageError<Self::DidWebvhResolverError>>
    where
        M: DeserializeOwned + Send,
    {
        async {
            let sign1 = coset::CoseSign1::from_slice(data).map_err(DecodeMessageError::Cose)?;
            let payload = sign1
                .payload
                .as_ref()
                .ok_or(DecodeMessageError::PayloadEmpty)?;
            let message: WithToken<M> =
                ciborium::from_reader(Cursor::new(payload)).map_err(DecodeMessageError::Cbor)?;

            let from_doc = self
                .resolve_identifier(&message.token.did)
                .await
                .map_err(DecodeMessageError::GetDidDocument)?
                .ok_or(DecodeMessageError::NotFoundPubkey)?;
            let public_key: VerifyingKey = from_doc.get_key("#signTimeSeriesKey")?.try_into()?;
            if message.token.exp < Utc::now() {
                return Err(DecodeMessageError::Expired);
            }
            sign1.verify_signature(b"", |sig, data| {
                sig.try_into()
                    .map_err(|_| DecodeMessageError::VecToArray(sig.len()))
                    .and_then(|sig| {
                        public_key
                            .verify(data, &Signature::from_bytes(sig))
                            .map_err(DecodeMessageError::Signature)
                    })
            })?;
            Ok(message)
        }
    }
}

impl<R: DidWebvhResolverService> MessageVerifier for R {}

// pub fn decode_message<M>(data: &[u8]) -> Result<(WithToken<M>, coset::CoseSign1), DecodeMessageError>
// where
//     M: DeserializeOwned
// {
//     let sign1 = coset::CoseSign1::from_slice(data).map_err(DecodeMessageError::Cose)?;
//     let payload = sign1
//         .payload
//         .as_ref()
//         .ok_or(DecodeMessageError::PayloadEmpty)?;
//     Ok((ciborium::from_reader(Cursor::new(payload)).map_err(DecodeMessageError::Cbor)?, sign1))
// }

// pub fn verify_message<M>(
//     from_doc: &DidDocument,
//     (message, sign1): (WithToken<M>, coset::CoseSign1)
// ) -> Result<WithToken<M>, DecodeMessageError>
// where
//     M: DeserializeOwned,
// {
//     let public_key: VerifyingKey = from_doc.get_key("#signTimeSeriesKey")?.try_into()?;
//     if message.token.exp < Utc::now() {
//         return Err(DecodeMessageError::Expired);
//     }
//     sign1.verify_signature(b"", |sig, data| {
//         sig.try_into()
//             .map_err(|_| DecodeMessageError::VecToArray(sig.len()))
//             .and_then(|sig| {
//                 public_key
//                     .verify(data, &Signature::from_bytes(sig))
//                     .map_err(DecodeMessageError::Signature)
//             })
//     })?;
//     Ok(message)
// }

// pub async fn verify_message<R, M>(
//     resolver: &mut R,
//     data: &[u8],
// ) -> Result<WithToken<M>, DecodeMessageError<R::DidWebvhResolverError>>
// where
//     R: DidWebvhResolverService,
//     M: DeserializeOwned,
// {
//     let sign1 = coset::CoseSign1::from_slice(data).map_err(DecodeMessageError::Cose)?;
//     let payload = sign1
//         .payload
//         .as_ref()
//         .ok_or(DecodeMessageError::PayloadEmpty)?;
//     let message: WithToken<M> =
//         ciborium::from_reader(Cursor::new(payload)).map_err(DecodeMessageError::Cbor)?;

//     let from_doc = resolver
//         .resolve_identifier(&message.token.did)
//         .await
//         .map_err(DecodeMessageError::GetDidDocument)?
//         .ok_or(DecodeMessageError::NotFoundPubkey)?;
//     let public_key: VerifyingKey = from_doc.get_key("#signTimeSeriesKey")?.try_into()?;
//     if message.token.exp < Utc::now() {
//         return Err(DecodeMessageError::Expired);
//     }
//     sign1.verify_signature(b"", |sig, data| {
//         sig.try_into()
//             .map_err(|_| DecodeMessageError::VecToArray(sig.len()))
//             .and_then(|sig| {
//                 public_key
//                     .verify(data, &Signature::from_bytes(sig))
//                     .map_err(DecodeMessageError::Signature)
//             })
//     })?;
//     Ok(message)
// }
