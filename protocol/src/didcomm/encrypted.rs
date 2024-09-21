use cuid;
pub use didcomm_rs;
use didcomm_rs::{crypto::CryptoAlgorithm, AttachmentBuilder, AttachmentDataBuilder, Message};
pub use serde_json;
use serde_json::Value;
use thiserror::Error;

use crate::{
    did::{
        did_repository::{get_encrypt_key, get_sign_key, DidRepository, GetPublicKeyError},
        sidetree::payload::DidDocument,
    },
    didcomm::types::{DidCommMessage, FindSenderError},
    keyring::keypair::{KeyPair, KeyPairing},
    verifiable_credentials::{
        credential_signer::{CredentialSigner, CredentialSignerVerifyError},
        did_vc::DidVcService,
        types::{VerifiableCredentials, VerifiedContainer},
    },
};

#[trait_variant::make(Send)]
pub trait DidCommEncryptedService: Sync {
    type GenerateError: std::error::Error;
    type VerifyError: std::error::Error;
    async fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &KeyPairing,
        to_did: &str,
        metadata: Option<&Value>,
    ) -> Result<DidCommMessage, Self::GenerateError>;
    async fn verify(
        &self,
        my_keyring: &KeyPairing,
        message: &DidCommMessage,
    ) -> Result<VerifiedContainer, Self::VerifyError>;
}

fn didcomm_generate<R: DidRepository, V: DidVcService>(
    body: &VerifiableCredentials,
    from_keyring: &KeyPairing,
    to_doc: &DidDocument,
    metadata: Option<&Value>,
    attachment_link: Option<&str>,
) -> Result<
    DidCommMessage,
    DidCommEncryptedServiceGenerateError<R::FindIdentifierError, V::GenerateError>,
> {
    let to_did = &to_doc.id;
    let from_did = &body.issuer.id;
    let body = serde_json::to_string(body)?;

    let mut message = Message::new().from(from_did).to(&[to_did]).body(&body)?;

    if let Some(value) = metadata {
        let id = cuid::cuid2();

        // let media_type = "application/json";
        let data = AttachmentDataBuilder::new().with_json(&value.to_string());

        let data = if let Some(attachment_link) = attachment_link {
            data.with_link(attachment_link)
        } else {
            data
        };

        message.append_attachment(
            AttachmentBuilder::new(true).with_id(&id).with_format("metadata").with_data(data),
        )
    }

    let public_key = get_encrypt_key(to_doc)?.as_bytes().to_vec();
    let public_key = Some(public_key);

    let seal_message = message
        .as_jwe(&CryptoAlgorithm::XC20P, public_key.clone())
        .seal(from_keyring.encrypt.get_secret_key().as_bytes(), Some(vec![public_key]))?;

    Ok(serde_json::from_str::<DidCommMessage>(&seal_message)?)
}

async fn generate<R: DidRepository, V: DidVcService>(
    did_repository: &R,
    vc_service: &V,
    model: VerifiableCredentials,
    from_keyring: &KeyPairing,
    to_did: &str,
    metadata: Option<&Value>,
    attachment_link: Option<&str>,
) -> Result<
    DidCommMessage,
    DidCommEncryptedServiceGenerateError<R::FindIdentifierError, V::GenerateError>,
> {
    let body = vc_service
        .generate(model, from_keyring)
        .map_err(DidCommEncryptedServiceGenerateError::VcService)?;
    let to_doc = did_repository
        .find_identifier(to_did)
        .await
        .map_err(DidCommEncryptedServiceGenerateError::SidetreeFindRequestFailed)?
        .ok_or(DidCommEncryptedServiceGenerateError::DidDocNotFound(to_did.to_string()))?
        .did_document;

    didcomm_generate::<R, V>(&body, from_keyring, &to_doc, metadata, attachment_link)
}

fn didcomm_verify<R: DidRepository>(
    from_doc: &DidDocument,
    my_keyring: &KeyPairing,
    message: &DidCommMessage,
) -> Result<VerifiedContainer, DidCommEncryptedServiceVerifyError<R::FindIdentifierError>> {
    let public_key = get_encrypt_key(from_doc)?.as_bytes().to_vec();
    let public_key = Some(public_key);

    let message = Message::receive(
        &serde_json::to_string(&message)?,
        Some(my_keyring.encrypt.get_secret_key().as_bytes().as_ref()),
        public_key,
        None,
    )?;

    let metadata = message.attachment_iter().find(|item| match &item.format {
        Some(value) => value == "metadata",
        None => false,
    });

    let body = message
        .get_body()
        .map_err(|e| DidCommEncryptedServiceVerifyError::MetadataBodyNotFound(Some(e)))?;
    let body = serde_json::from_str::<VerifiableCredentials>(&body)?;

    match metadata {
        Some(metadata) => {
            let metadata = metadata
                .data
                .json
                .as_ref()
                .ok_or(DidCommEncryptedServiceVerifyError::MetadataBodyNotFound(None))?;
            let metadata = serde_json::from_str::<Value>(metadata)?;
            Ok(VerifiedContainer { message: body, metadata: Some(metadata) })
        }
        None => Ok(VerifiedContainer { message: body, metadata: None }),
    }
}

async fn verify<R: DidRepository>(
    did_repository: &R,
    my_keyring: &KeyPairing,
    message: &DidCommMessage,
) -> Result<VerifiedContainer, DidCommEncryptedServiceVerifyError<R::FindIdentifierError>> {
    let other_did = message.find_sender()?;
    let other_doc = did_repository
        .find_identifier(&other_did)
        .await
        .map_err(DidCommEncryptedServiceVerifyError::SidetreeFindRequestFailed)?
        .ok_or(DidCommEncryptedServiceVerifyError::DidDocNotFound(other_did))?
        .did_document;
    let mut container = didcomm_verify::<R>(&other_doc, my_keyring, message)?;
    // For performance, call low level api
    let public_key = get_sign_key(&other_doc)?;
    let body = CredentialSigner::verify(container.message, &public_key)?;
    container.message = body;
    Ok(container)
}

#[derive(Debug, Error)]
pub enum DidCommEncryptedServiceGenerateError<FindIdentifierError, CredentialSignerSignError>
where
    FindIdentifierError: std::error::Error,
    CredentialSignerSignError: std::error::Error,
{
    #[error("failed to get did document: {0}")]
    DidDocNotFound(String),
    #[error("did public key not found. did: {0}")]
    DidPublicKeyNotFound(#[from] GetPublicKeyError),
    #[error("something went wrong with vc service: {0}")]
    VcService(CredentialSignerSignError),
    #[error("failed to create identifier: {0}")]
    SidetreeFindRequestFailed(FindIdentifierError),
    #[error("failed to encrypt message with error: {0}")]
    EncryptFailed(#[from] didcomm_rs::Error),
    #[error("failed serialize/deserialize: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum DidCommEncryptedServiceVerifyError<FindIdentifierError: std::error::Error> {
    #[error("failed to get did document: {0}")]
    DidDocNotFound(String),
    #[error("something went wrong with vc service: {0}")]
    VcService(#[from] CredentialSignerVerifyError),
    #[error("failed to find identifier: {0}")]
    SidetreeFindRequestFailed(FindIdentifierError),
    #[error("did public key not found. did: {0}")]
    DidPublicKeyNotFound(#[from] GetPublicKeyError),
    #[error("failed to decrypt message: {0:?}")]
    DecryptFailed(#[from] didcomm_rs::Error),
    #[error("failed to get body: {0:?}")]
    MetadataBodyNotFound(Option<didcomm_rs::Error>),
    #[error("failed serialize/deserialize: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to find sender did: {0}")]
    FindSender(#[from] FindSenderError),
}

impl<R> DidCommEncryptedService for R
where
    R: DidRepository + DidVcService,
{
    type GenerateError =
        DidCommEncryptedServiceGenerateError<R::FindIdentifierError, R::GenerateError>;
    type VerifyError = DidCommEncryptedServiceVerifyError<R::FindIdentifierError>;
    async fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &KeyPairing,
        to_did: &str,
        metadata: Option<&Value>,
    ) -> Result<DidCommMessage, Self::GenerateError> {
        generate::<R, R>(self, self, model, from_keyring, to_did, metadata, None).await
    }

    async fn verify(
        &self,
        my_keyring: &KeyPairing,
        message: &DidCommMessage,
    ) -> Result<VerifiedContainer, Self::VerifyError> {
        verify(self, my_keyring, message).await
    }
}

pub struct DidCommServiceWithAttachment<R>
where
    R: DidRepository + DidVcService,
{
    vc_service: R,
    attachment_link: String,
}

impl<R> DidCommServiceWithAttachment<R>
where
    R: DidRepository + DidVcService,
{
    pub fn new(did_repository: R, attachment_link: String) -> Self {
        Self { vc_service: did_repository, attachment_link }
    }
}

impl<R> DidCommEncryptedService for DidCommServiceWithAttachment<R>
where
    R: DidRepository + DidVcService,
{
    type GenerateError =
        DidCommEncryptedServiceGenerateError<R::FindIdentifierError, R::GenerateError>;
    type VerifyError = DidCommEncryptedServiceVerifyError<R::FindIdentifierError>;
    async fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &KeyPairing,
        to_did: &str,
        metadata: Option<&Value>,
    ) -> Result<DidCommMessage, Self::GenerateError> {
        generate::<R, R>(
            &self.vc_service,
            &self.vc_service,
            model,
            from_keyring,
            to_did,
            metadata,
            Some(&self.attachment_link),
        )
        .await
    }

    async fn verify(
        &self,
        my_keyring: &KeyPairing,
        message: &DidCommMessage,
    ) -> Result<VerifiedContainer, Self::VerifyError> {
        verify(&self.vc_service, my_keyring, message).await
    }
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

    #[actix_rt::test]
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
        let res = repo.generate(model, &from_keyring, &to_did, None).await.unwrap();

        let verified = repo.verify(&to_keyring, &res).await.unwrap();
        let verified = verified.message;

        assert_eq!(verified.issuer.id, from_did);
        assert_eq!(verified.credential_subject.container, message);
    }

    mod generate_failed {
        use super::*;
        use crate::did::did_repository::mocks::NoPublicKeyDidRepository;

        #[actix_rt::test]
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
            let res = repo.generate(model, &from_keyring, &to_did, None).await.unwrap_err();

            if let DidCommEncryptedServiceGenerateError::DidDocNotFound(did) = res {
                assert_eq!(did, to_did);
            } else {
                panic!("unexpected result: {:?}", res);
            }
        }

        #[actix_rt::test]
        async fn test_did_public_key_not_found() {
            let from_did = create_random_did();
            let to_did = create_random_did();

            let from_keyring = KeyPairing::create_keyring(OsRng);

            let repo = NoPublicKeyDidRepository;

            let message = json!({"test": "0123456789abcdef"});
            let issuance_date = Utc::now();

            let model = VerifiableCredentials::new(from_did, message, issuance_date);
            let res = repo.generate(model, &from_keyring, &to_did, None).await.unwrap_err();

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

            repo.generate(model, from_keyring, to_did, metadata).await.unwrap()
        }

        #[actix_rt::test]
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

        #[actix_rt::test]
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

        #[actix_rt::test]
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
