use crate::{
    repository::did_repository::DidRepository,
    services::{internal::did_vc::DIDVCService, project_verifier::ProjectVerifier},
};
use anyhow::Context;
use chrono::DateTime;
use chrono::Utc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

pub struct VerifiableMessageUseCase {
    project_verifier: Box<dyn ProjectVerifier>,
    did_repository: Box<dyn DidRepository>,
    vc_service: DIDVCService,
}

impl VerifiableMessageUseCase {
    pub fn new(
        project_verifier: Box<dyn ProjectVerifier>,
        did_repository: Box<dyn DidRepository>,
        vc_service: DIDVCService,
    ) -> Self {
        Self {
            project_verifier,
            did_repository,
            vc_service,
        }
    }
}

#[derive(Debug, Error)]
pub enum CreateVerifiableMessageUseCaseError {
    #[error("destination did not found")]
    DestinationNotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum VerifyVerifiableMessageUseCaseError {
    #[error("verification failed")]
    VerificationFailed,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl VerifiableMessageUseCase {
    pub async fn generate(
        &self,
        destination_did: String,
        message: String,
        now: DateTime<Utc>,
    ) -> Result<String, CreateVerifiableMessageUseCaseError> {
        self.did_repository
            .find_identifier(&destination_did)
            .await?
            .ok_or(CreateVerifiableMessageUseCaseError::DestinationNotFound)?;

        let message = EncodedMessage {
            message_id: Uuid::new_v4().to_string(),
            payload: message,
            destination_did: destination_did.clone(),
            created_at: now.to_rfc3339(),
            project_hmac: self.project_verifier.create_project_hmac()?,
        };

        let message = serde_json::to_value(message).context("failed to convert to value")?;
        let vc = DIDVCService::generate(&self.vc_service, &message, now)?;

        Ok(serde_json::to_string(&vc).context("failed to serialize")?)
    }

    #[allow(dead_code)]
    pub async fn verify(
        &self,
        message: &str,
    ) -> Result<String, VerifyVerifiableMessageUseCaseError> {
        let message = serde_json::from_str::<Value>(message).context("failed to decode str")?;

        let vc = DIDVCService::verify(&self.vc_service, &message)
            .await
            .context("verify failed")?;

        let container = vc
            .get("credentialSubject")
            .context("credentialSubject not found")?
            .get("container")
            .context("container not found")?
            .clone();

        let message = serde_json::from_value::<EncodedMessage>(container)
            .context("failed to deserialize to EncodedMessage")?;

        if self
            .project_verifier
            .verify_project_hmac(&message.project_hmac)?
        {
            Ok(message.payload)
        } else {
            Err(VerifyVerifiableMessageUseCaseError::VerificationFailed)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodedMessage {
    pub message_id: String,
    pub payload: String,
    pub destination_did: String,
    pub created_at: String,
    pub project_hmac: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        nodex::keyring::keypair::KeyPairing,
        nodex::sidetree::payload::{
            DIDDocument, DIDResolutionResponse, DidPublicKey, MethodMetadata,
        },
        services::project_verifier::ProjectVerifier,
    };

    struct MockProjectVerifier {}

    impl ProjectVerifier for MockProjectVerifier {
        fn create_project_hmac(&self) -> anyhow::Result<String> {
            Ok("mock".to_string())
        }

        fn verify_project_hmac(&self, _signature: &str) -> anyhow::Result<bool> {
            Ok(true)
        }
    }

    struct MockDidRepository {}

    #[async_trait::async_trait]
    impl DidRepository for MockDidRepository {
        async fn create_identifier(&self) -> anyhow::Result<DIDResolutionResponse> {
            // DID doesn't matter
            let did = "did:example:123";

            let mut keyring = KeyPairing::create_keyring()?;
            keyring.save(did);

            self.find_identifier(&did)
                .await
                .and_then(|x| x.context("unreachable"))
        }
        async fn find_identifier(
            &self,
            did: &str,
        ) -> anyhow::Result<Option<DIDResolutionResponse>> {
            // extract from NodeX::create_identifier
            let jwk = KeyPairing::load_keyring()?
                .get_sign_key_pair()
                .to_jwk(false)?;

            Ok(Some(DIDResolutionResponse {
                context: "https://www.w3.org/ns/did-resolution/v1".to_string(),
                did_document: DIDDocument {
                    id: did.to_string(),
                    public_key: Some(vec![DidPublicKey {
                        id: did.to_string() + "#signingKey",
                        controller: String::new(),
                        r#type: "EcdsaSecp256k1VerificationKey2019".to_string(),
                        public_key_jwk: jwk,
                    }]),
                    service: None,
                    authentication: Some(vec!["signingKey".to_string()]),
                },
                method_metadata: MethodMetadata {
                    published: true,
                    recovery_commitment: None,
                    update_commitment: None,
                },
            }))
        }
    }

    #[tokio::test]
    async fn test_verifiable_message_usecase() {
        // generate local did and keys
        let repository = MockDidRepository {};
        repository.create_identifier().await.unwrap();

        let usecase = VerifiableMessageUseCase {
            project_verifier: Box::new(MockProjectVerifier {}),
            did_repository: Box::new(MockDidRepository {}),
            vc_service: DIDVCService::new(MockDidRepository {}),
        };

        let destination_did = "did:example:123".to_string();
        let message = "Hello".to_string();

        let now = Utc::now();
        let generated = usecase
            .generate(destination_did, message.clone(), now)
            .await
            .unwrap();

        let result: Value = serde_json::from_str(&generated).unwrap();
        dbg!(&result);

        let message_id = result["credentialSubject"]["container"]["message_id"]
            .as_str()
            .unwrap();

        assert_eq!(
            result["credentialSubject"]["container"],
            serde_json::json!({
                "message_id": message_id,
                "payload": "Hello",
                "destination_did": "did:example:123",
                "created_at": now.to_rfc3339(),
                "project_hmac": "mock"
            })
        );

        let verified = usecase.verify(&generated).await.unwrap();
        assert_eq!(verified, message);
    }
}
