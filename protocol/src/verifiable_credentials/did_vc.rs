use thiserror::Error;

use crate::{
    did::did_repository::{get_sign_key, DidRepository, GetPublicKeyError},
    keyring::keypair,
    verifiable_credentials::{
        credential_signer::{
            CredentialSigner, CredentialSignerSignError, CredentialSignerSuite,
            CredentialSignerVerifyError,
        },
        types::VerifiableCredentials,
    },
};

#[trait_variant::make(Send)]
pub trait DidVcService: Sync {
    type GenerateError: std::error::Error + Send + Sync;
    type VerifyError: std::error::Error + Send + Sync;
    fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &keypair::KeyPairing,
    ) -> Result<VerifiableCredentials, Self::GenerateError>;
    async fn verify(
        &self,
        model: VerifiableCredentials,
    ) -> Result<VerifiableCredentials, Self::VerifyError>;
}

#[derive(Debug, Error)]
pub enum DidVcServiceVerifyError<FindIdentifierError: std::error::Error> {
    #[error("did public key not found. did: {0}")]
    PublicKeyNotFound(#[from] GetPublicKeyError),
    #[error("failed to get did document: {0}")]
    DidDocNotFound(String),
    #[error("failed to find identifier: {0}")]
    FindIdentifier(FindIdentifierError),
    #[error("credential signer error")]
    VerifyFailed(#[from] CredentialSignerVerifyError),
}

impl<R: DidRepository> DidVcService for R {
    type GenerateError = CredentialSignerSignError;
    type VerifyError = DidVcServiceVerifyError<R::FindIdentifierError>;
    fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &keypair::KeyPairing,
    ) -> Result<VerifiableCredentials, Self::GenerateError> {
        let did = &model.issuer.id.clone();
        CredentialSigner::sign(
            model,
            CredentialSignerSuite { did, key_id: "signingKey", context: &from_keyring.sign },
        )
    }

    async fn verify(
        &self,
        model: VerifiableCredentials,
    ) -> Result<VerifiableCredentials, Self::VerifyError> {
        let did_document = self
            .find_identifier(&model.issuer.id)
            .await
            .map_err(Self::VerifyError::FindIdentifier)?;
        let did_document = did_document
            .ok_or(DidVcServiceVerifyError::DidDocNotFound(model.issuer.id.clone()))?
            .did_document;
        let public_key = get_sign_key(&did_document)?;
        Ok(CredentialSigner::verify(model, &public_key)?)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, iter::FromIterator as _};

    use chrono::{DateTime, Utc};
    use rand_core::OsRng;
    use serde_json::{json, Value};

    use super::{DidVcService, DidVcServiceVerifyError, VerifiableCredentials};
    use crate::{
        did::{did_repository::mocks::MockDidRepository, test_utils::create_random_did},
        keyring::keypair::KeyPairing,
    };

    #[actix_rt::test]
    async fn test_generate_and_verify() {
        let from_did = create_random_did();

        let from_keyring = KeyPairing::create_keyring(OsRng);

        let mock_repository = MockDidRepository::from_single(BTreeMap::from_iter([(
            from_did.clone(),
            from_keyring.clone(),
        )]));

        let service = mock_repository;

        let message = json!({"test": "0123456789abcdef"});
        let issuance_date = Utc::now();

        let model = VerifiableCredentials::new(from_did.clone(), message.clone(), issuance_date);
        let res = service.generate(model, &from_keyring).unwrap();

        let verified = service.verify(res).await.unwrap();

        assert_eq!(verified.issuer.id, from_did);
        assert_eq!(verified.credential_subject.container, message);
    }

    mod generate_failed {}

    mod verify_failed {
        use super::*;
        use crate::did::did_repository::mocks::{
            IllegalPublicKeyLengthDidRepository, NoPublicKeyDidRepository,
        };

        fn create_did_vc(
            from_did: &str,
            from_keyring: &KeyPairing,
            message: &Value,
            issuance_date: DateTime<Utc>,
        ) -> VerifiableCredentials {
            let service = MockDidRepository::from_single(BTreeMap::new());
            let model =
                VerifiableCredentials::new(from_did.to_string(), message.clone(), issuance_date);
            service.generate(model, from_keyring).unwrap()
        }

        #[actix_rt::test]
        async fn test_did_not_found() {
            let from_did = create_random_did();

            let mock_repository = MockDidRepository::from_single(BTreeMap::new());

            let service = mock_repository;

            let model = create_did_vc(
                &from_did,
                &KeyPairing::create_keyring(OsRng),
                &json!({}),
                Utc::now(),
            );

            let res = service.verify(model).await.unwrap_err();

            if let DidVcServiceVerifyError::DidDocNotFound(_) = res {
            } else {
                panic!("unexpected error: {:?}", res);
            }
        }

        #[actix_rt::test]
        async fn test_public_key_not_found() {
            let from_did = create_random_did();

            let model = create_did_vc(
                &from_did,
                &KeyPairing::create_keyring(OsRng),
                &json!({}),
                Utc::now(),
            );

            let mock_repository = NoPublicKeyDidRepository;
            let service = mock_repository;

            let res = service.verify(model).await.unwrap_err();

            if let DidVcServiceVerifyError::PublicKeyNotFound(_) = res {
            } else {
                panic!("unexpected error: {:?}", res);
            }
        }

        #[actix_rt::test]
        async fn test_verify_failed() {
            let from_did = create_random_did();

            let mut model = create_did_vc(
                &from_did,
                &KeyPairing::create_keyring(OsRng),
                &json!({}),
                Utc::now(),
            );
            // for failing credential signer
            model.proof = None;

            let mock_repository = MockDidRepository::from_single(BTreeMap::from_iter([(
                from_did.clone(),
                KeyPairing::create_keyring(OsRng),
            )]));
            let service = mock_repository;

            let res = service.verify(model).await.unwrap_err();

            if let DidVcServiceVerifyError::VerifyFailed(_) = res {
            } else {
                panic!("unexpected error: {:?}", res);
            }
        }

        #[actix_rt::test]
        async fn test_public_key_length_mismatch() {
            let from_did = create_random_did();

            let model = create_did_vc(
                &from_did,
                &KeyPairing::create_keyring(OsRng),
                &json!({}),
                Utc::now(),
            );

            let mock_repository = IllegalPublicKeyLengthDidRepository;
            let service = mock_repository;

            let res = service.verify(model).await.unwrap_err();

            if let DidVcServiceVerifyError::PublicKeyNotFound(_) = res {
            } else {
                panic!("unexpected error: {:?}", res);
            }
        }

        #[actix_rt::test]
        async fn test_signature_not_verified() {
            let from_did = create_random_did();

            let model = create_did_vc(
                &from_did,
                &KeyPairing::create_keyring(OsRng),
                &json!({}),
                Utc::now(),
            );

            let mock_repository = MockDidRepository::from_single(BTreeMap::from_iter([(
                from_did.clone(),
                KeyPairing::create_keyring(OsRng),
            )]));
            let service = mock_repository;

            let res = service.verify(model).await.unwrap_err();

            if let DidVcServiceVerifyError::VerifyFailed(_) = res {
            } else {
                panic!("unexpected error: {:?}", res);
            }
        }
    }
}
