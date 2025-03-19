use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use protocol::{
    didcomm::{encrypted::DidCommEncryptedService, types::DidCommMessage},
    verifiable_credentials::types::VerifiableCredentials,
};

use crate::{
    nodex::utils::did_accessor::DidAccessor,
    repository::message_activity_repository::{
        CreatedMessageActivityRequest, MessageActivityRepository, VerifiedMessageActivityRequest,
        VerifiedStatus,
    },
};

pub struct DidcommMessageUseCase<R, D, A>
where
    R: MessageActivityRepository,
    D: DidCommEncryptedService,
    A: DidAccessor,
{
    message_activity_repository: R,
    didcomm_service: D,
    did_accessor: A,
}

#[derive(Debug, Error)]
pub enum GenerateDidcommMessageUseCaseError<E, F>
where
    E: std::error::Error,
    F: std::error::Error,
{
    #[error("encrypted service error: {0}")]
    ServiceGenerate(E),
    #[error("message activity error: {0}")]
    MessageActivity(F),
    #[error("failed serialize/deserialize : {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum VerifyDidcommMessageUseCaseError<E, F>
where
    E: std::error::Error,
    F: std::error::Error,
{
    #[error("encrypted service error: {0}")]
    ServiceVerify(E),
    //TODO: Add tests
    #[error("This message is not addressed to me")]
    NotAddressedToMe,
    #[error("message activity error: {0}")]
    MessageActivity(F),
    #[error("failed serialize/deserialize : {0}")]
    Json(#[from] serde_json::Error),
}

impl<R, D, A> DidcommMessageUseCase<R, D, A>
where
    R: MessageActivityRepository,
    D: DidCommEncryptedService,
    A: DidAccessor,
{
    pub fn new(message_activity_repository: R, didcomm_service: D, did_accessor: A) -> Self {
        DidcommMessageUseCase {
            message_activity_repository,
            didcomm_service,
            did_accessor,
        }
    }

    pub async fn generate(
        &self,
        destination_did: String,
        message: String,
        operation_tag: String,
        now: DateTime<Utc>,
    ) -> Result<String, GenerateDidcommMessageUseCaseError<D::GenerateError, R::Error>> {
        let message_id = Uuid::new_v4();

        let message = EncodedMessage {
            message_id,
            payload: message,
            created_at: now.to_rfc3339(),
        };
        let message = serde_json::to_value(message)?;
        let my_did = self.did_accessor.get_my_did();

        let model = VerifiableCredentials::new(my_did.clone().into_inner(), message, now);
        let didcomm_message = self
            .didcomm_service
            .generate(
                model,
                &self.did_accessor.get_my_keyring(),
                &destination_did,
                None,
            )
            .await
            .map_err(GenerateDidcommMessageUseCaseError::ServiceGenerate)?;

        let result = serde_json::to_string(&didcomm_message)?;

        self.message_activity_repository
            .add_create_activity(CreatedMessageActivityRequest {
                message_id,
                from: my_did.into_inner(),
                to: destination_did,
                operation_tag,
                is_encrypted: true,
                occurred_at: now,
            })
            .await
            .map_err(GenerateDidcommMessageUseCaseError::MessageActivity)?;

        Ok(result)
    }

    pub async fn verify(
        &self,
        message: DidCommMessage,
        now: DateTime<Utc>,
    ) -> Result<VerifiableCredentials, VerifyDidcommMessageUseCaseError<D::VerifyError, R::Error>>
    {
        let my_did = self.did_accessor.get_my_did();
        if !message
            .find_receivers()
            .contains(&my_did.clone().into_inner())
        {
            return Err(VerifyDidcommMessageUseCaseError::NotAddressedToMe);
        }
        let verified = self
            .didcomm_service
            .verify(&self.did_accessor.get_my_keyring(), &message)
            .await
            .map_err(VerifyDidcommMessageUseCaseError::ServiceVerify)?;
        let verified = verified.message;
        let from_did = verified.issuer.id.clone();
        // check in verified. maybe exists?
        let container = verified.clone().credential_subject.container;
        let message = serde_json::from_value::<EncodedMessage>(container)?;

        self.message_activity_repository
            .add_verify_activity(VerifiedMessageActivityRequest {
                from: from_did,
                to: my_did.into_inner(),
                message_id: message.message_id,
                verified_at: now,
                status: VerifiedStatus::Valid,
            })
            .await
            .map_err(VerifyDidcommMessageUseCaseError::MessageActivity)?;

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

    use protocol::didcomm::encrypted::DidCommEncryptedServiceGenerateError;
    use protocol::didcomm::encrypted::DidCommEncryptedServiceVerifyError;

    use crate::nodex::utils::did_accessor::mocks::MockDidAccessor;
    use crate::repository::did_repository::mocks::MockDidRepository;
    use crate::repository::message_activity_repository::mocks::MockMessageActivityRepository;
    use crate::usecase::test_util::TestPresets;

    use super::*;

    #[tokio::test]
    async fn test_create_and_verify() {
        let presets = TestPresets::default();
        let repo = presets.create_mock_did_repository();
        let usecase = DidcommMessageUseCase::new(
            MockMessageActivityRepository::create_success(),
            repo.clone(),
            MockDidAccessor::new(presets.from_did, presets.from_keyring),
        );

        let message = "Hello".to_string();

        let now = Utc::now();
        let generated = usecase
            .generate(
                presets.to_did.clone(),
                message.clone(),
                "test".to_string(),
                now,
            )
            .await
            .unwrap();
        let generated = serde_json::from_str::<DidCommMessage>(&generated).unwrap();

        let usecase = DidcommMessageUseCase::new(
            MockMessageActivityRepository::verify_success(),
            repo,
            MockDidAccessor::new(presets.to_did, presets.to_keyring),
        );

        let verified = usecase.verify(generated, Utc::now()).await.unwrap();
        let encoded_message =
            serde_json::from_value::<EncodedMessage>(verified.credential_subject.container)
                .unwrap();
        assert_eq!(encoded_message.payload, message);
    }

    mod generate_failed {
        use crate::nodex::utils::did_accessor::mocks::MockDidAccessor;

        use super::*;

        #[tokio::test]
        async fn test_generate_did_not_found() {
            let presets = TestPresets::default();

            let usecase = DidcommMessageUseCase::new(
                MockMessageActivityRepository::create_success(),
                MockDidRepository::empty(),
                MockDidAccessor::new(presets.from_did, presets.from_keyring),
            );

            let message = "Hello".to_string();

            let now = Utc::now();
            let generated = usecase
                .generate(presets.to_did.clone(), message, "test".to_string(), now)
                .await;

            if let Err(GenerateDidcommMessageUseCaseError::ServiceGenerate(
                DidCommEncryptedServiceGenerateError::DidDocNotFound(_),
            )) = generated
            {
            } else {
                panic!("unexpected result: {:?}", generated);
            }
        }

        #[tokio::test]
        async fn test_generate_add_activity_failed() {
            let presets = TestPresets::default();

            let usecase = DidcommMessageUseCase::new(
                MockMessageActivityRepository::create_fail(),
                presets.create_mock_did_repository(),
                MockDidAccessor::new(presets.from_did, presets.from_keyring),
            );

            let message = "Hello".to_string();

            let now = Utc::now();
            let generated = usecase
                .generate(presets.to_did.clone(), message, "test".to_string(), now)
                .await;

            if let Err(GenerateDidcommMessageUseCaseError::MessageActivity(_)) = generated {
            } else {
                panic!("unexpected result: {:?}", generated);
            }
        }
    }

    mod verify_failed {
        use super::*;
        use crate::nodex::utils::did_accessor::mocks::MockDidAccessor;

        async fn create_test_message_for_verify_test(presets: TestPresets) -> String {
            let usecase = DidcommMessageUseCase::new(
                MockMessageActivityRepository::create_success(),
                presets.create_mock_did_repository(),
                MockDidAccessor::new(presets.from_did, presets.from_keyring),
            );

            let message = "Hello".to_string();

            let now = Utc::now();

            usecase
                .generate(
                    presets.to_did.clone(),
                    message.clone(),
                    "test".to_string(),
                    now,
                )
                .await
                .unwrap()
        }

        #[tokio::test]
        async fn test_verify_did_not_found() {
            let presets = TestPresets::default();
            let generated = create_test_message_for_verify_test(presets.clone()).await;
            let generated = serde_json::from_str::<DidCommMessage>(&generated).unwrap();

            let usecase = DidcommMessageUseCase::new(
                MockMessageActivityRepository::verify_success(),
                MockDidRepository::empty(),
                MockDidAccessor::new(presets.to_did, presets.to_keyring),
            );

            let verified = usecase.verify(generated, Utc::now()).await;

            if let Err(VerifyDidcommMessageUseCaseError::ServiceVerify(
                DidCommEncryptedServiceVerifyError::DidDocNotFound(_),
            )) = verified
            {
            } else {
                panic!("unexpected result: {:#?}", verified);
            }
        }

        #[tokio::test]
        async fn test_verify_add_activity_failed() {
            let presets = TestPresets::default();
            let generated = create_test_message_for_verify_test(presets.clone()).await;
            let generated = serde_json::from_str::<DidCommMessage>(&generated).unwrap();

            let usecase = DidcommMessageUseCase::new(
                MockMessageActivityRepository::verify_fail(),
                presets.create_mock_did_repository(),
                MockDidAccessor::new(presets.to_did, presets.to_keyring),
            );

            let verified = usecase.verify(generated, Utc::now()).await;

            if let Err(VerifyDidcommMessageUseCaseError::MessageActivity(_)) = verified {
            } else {
                panic!("unexpected result: {:?}", verified);
            }
        }
    }
}
