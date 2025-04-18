use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use protocol::{
    did_webvh::{
        domain::did::{Did, DidError},
        service::resolver::resolver_service::DidWebvhResolverService,
    },
    didcomm::{
        sign_encrypt::{
            decrypt_message, encrypt_message, DidCommDecryptMessageError,
            DidCommEncryptMessageError,
        },
        types::{DidCommMessage, FindSenderError},
    },
};

use crate::nodex::utils::did_accessor::DidAccessor;

pub struct DidcommMessageUseCase<D, A>
where
    D: DidWebvhResolverService,
    A: DidAccessor,
{
    webvh: D,
    did_accessor: A,
}

#[derive(Debug, Error)]
pub enum GenerateDidcommMessageUseCaseError {
    #[error("Failed to convert did: {0}")]
    InvalidDid(#[from] DidError),
    #[error("Failed to generate message: {0}")]
    Generate(#[from] DidCommEncryptMessageError),
    #[error("failed serialize/deserialize : {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum VerifyDidcommMessageUseCaseError {
    #[error("Failed to verify message: {0}")]
    Verify(#[from] DidCommDecryptMessageError),
    #[error("Failed to find sender: {0}")]
    FindSender(#[from] FindSenderError),
    #[error("failed serialize/deserialize : {0}")]
    Json(#[from] serde_json::Error),
    #[error("This message is not addressed to me")]
    NotAddressedToMe,
}

impl<D, A> DidcommMessageUseCase<D, A>
where
    D: DidWebvhResolverService,
    A: DidAccessor,
{
    pub fn new(webvh: D, did_accessor: A) -> Self {
        DidcommMessageUseCase {
            webvh,
            did_accessor,
        }
    }

    pub async fn generate(
        &mut self,
        destination_did: String,
        message: String,
        now: DateTime<Utc>,
    ) -> Result<String, GenerateDidcommMessageUseCaseError> {
        let message_id = Uuid::new_v4();

        let message = EncodedMessage {
            message_id,
            payload: message,
            created_at: now.to_rfc3339(),
        };
        let message = serde_json::to_value(message)?;
        let my_did = self.did_accessor.get_my_did();

        let to_did = destination_did.parse::<Did>()?;
        let did_doc = self.webvh.resolve_identifier(&to_did).await.map_err(|e| {
            GenerateDidcommMessageUseCaseError::Generate(
                DidCommEncryptMessageError::DidDocNotFound(e.to_string()),
            )
        })?;

        let didcomm_message = encrypt_message(
            &message.to_string(),
            &my_did,
            &self.did_accessor.get_my_keyring(),
            &did_doc.ok_or_else(|| {
                GenerateDidcommMessageUseCaseError::Generate(
                    DidCommEncryptMessageError::DidDocNotFound("did doc not found".to_string()),
                )
            })?,
        )?;

        let result = serde_json::to_string(&didcomm_message)?;

        Ok(result)
    }

    pub async fn verify(
        &mut self,
        message: DidCommMessage,
    ) -> Result<String, VerifyDidcommMessageUseCaseError> {
        let my_did = self.did_accessor.get_my_did();
        if !message
            .find_receivers()
            .contains(&my_did.clone().into_inner())
        {
            log::error!(
                "message not addressed to me: {}, contains: {:?}",
                my_did,
                message.find_receivers()
            );
            return Err(VerifyDidcommMessageUseCaseError::NotAddressedToMe);
        }

        let sender_did = message.find_sender()?;
        let from_doc = self
            .webvh
            .resolve_identifier(&sender_did)
            .await
            .map_err(|e| {
                VerifyDidcommMessageUseCaseError::Verify(
                    DidCommDecryptMessageError::DidDocNotFound(e.to_string()),
                )
            })?;
        let from_doc = from_doc.ok_or_else(|| {
            VerifyDidcommMessageUseCaseError::Verify(DidCommDecryptMessageError::DidDocNotFound(
                sender_did.to_string(),
            ))
        })?;
        let verified = decrypt_message(&message, &from_doc, &self.did_accessor.get_my_keyring())?;

        Ok(verified)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct EncodedMessage {
    pub message_id: Uuid,
    pub payload: String,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use serde_json;

    use crate::nodex::utils::did_accessor::mocks::MockDidAccessor;
    use crate::nodex::utils::mock_webvh_resover::mocks::MockDidWebvhResolverService;
    use crate::usecase::test_util::TestPresets;

    use super::*;

    #[tokio::test]
    async fn test_create_and_verify() {
        let presets = TestPresets::default();
        let mut usecase = DidcommMessageUseCase::new(
            MockDidWebvhResolverService::new(presets.to_did.clone(), presets.to_keyring.clone()),
            MockDidAccessor::new(presets.from_did.clone(), presets.from_keyring.clone()),
        );

        let message = "Hello".to_string();

        let now = Utc::now();
        let generated = usecase
            .generate(presets.to_did.clone(), message.clone(), now)
            .await
            .unwrap();
        let generated = serde_json::from_str::<DidCommMessage>(&generated).unwrap();

        let mut usecase = DidcommMessageUseCase::new(
            MockDidWebvhResolverService::new(presets.from_did, presets.from_keyring),
            MockDidAccessor::new(presets.to_did, presets.to_keyring),
        );

        let verified = usecase.verify(generated).await.unwrap();
        let verified = serde_json::from_str::<EncodedMessage>(&verified).unwrap();
        assert_eq!(verified.payload, message);
    }

    mod generate_failed {
        use crate::nodex::utils::did_accessor::mocks::MockDidAccessor;

        use super::*;

        #[tokio::test]
        async fn test_generate_did_not_found() {
            let presets = TestPresets::default();

            let mut usecase = DidcommMessageUseCase::new(
                MockDidWebvhResolverService::empty(),
                MockDidAccessor::new(presets.from_did, presets.from_keyring),
            );

            let message = "Hello".to_string();

            let now = Utc::now();
            let generated = usecase.generate(presets.to_did.clone(), message, now).await;

            if let Err(GenerateDidcommMessageUseCaseError::Generate(
                DidCommEncryptMessageError::DidDocNotFound(_),
            )) = generated
            {
            } else {
                panic!("unexpected result: {:?}", generated);
            }
        }
    }

    mod verify_failed {
        use super::*;
        use crate::nodex::utils::did_accessor::mocks::MockDidAccessor;

        async fn create_test_message_for_verify_test(presets: TestPresets) -> String {
            let mut usecase = DidcommMessageUseCase::new(
                MockDidWebvhResolverService::new(presets.to_did.clone(), presets.to_keyring),
                MockDidAccessor::new(presets.from_did, presets.from_keyring),
            );

            let message = "Hello".to_string();

            let now = Utc::now();

            usecase
                .generate(presets.to_did.clone(), message.clone(), now)
                .await
                .unwrap()
        }

        #[tokio::test]
        async fn test_verify_did_not_found() {
            let presets = TestPresets::default();
            let generated = create_test_message_for_verify_test(presets.clone()).await;
            let generated = serde_json::from_str::<DidCommMessage>(&generated).unwrap();

            let mut usecase = DidcommMessageUseCase::new(
                MockDidWebvhResolverService::empty(),
                MockDidAccessor::new(presets.to_did, presets.to_keyring),
            );

            let verified = usecase.verify(generated).await;

            dbg!(&verified);
            if let Err(VerifyDidcommMessageUseCaseError::Verify(_)) = verified {
            } else {
                panic!("unexpected result: {:?}", verified);
            }
        }
    }
}
