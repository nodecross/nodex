use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use nodex_didcomm::{
    did::did_repository::DidRepository,
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
    D: DidRepository + DidCommEncryptedService,
    A: DidAccessor,
{
    message_activity_repository: R,
    did_repository: D,
    did_accessor: A,
}

#[derive(Debug, Error)]
pub enum GenerateDidcommMessageUseCaseError<E, F>
where
    E: std::error::Error,
    F: std::error::Error,
{
    #[error("encrypted service error: {0}")]
    DidCommEncryptedServiceGenerateError(E),
    #[error("message activity error: {0}")]
    MessageActivityHttpError(F),
    #[error("failed serialize/deserialize : {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum VerifyDidcommMessageUseCaseError<E, F>
where
    E: std::error::Error,
    F: std::error::Error,
{
    #[error("encrypted service error: {0}")]
    DidCommEncryptedServiceVerifyError(E),
    #[error("message activity error: {0}")]
    MessageActivityHttpError(F),
    #[error("failed serialize/deserialize : {0}")]
    JsonError(#[from] serde_json::Error),
}

impl<R, D, A> DidcommMessageUseCase<R, D, A>
where
    R: MessageActivityRepository,
    D: DidRepository + DidCommEncryptedService,
    A: DidAccessor,
{
    pub fn new(message_activity_repository: R, did_repository: D, did_accessor: A) -> Self {
        DidcommMessageUseCase {
            message_activity_repository,
            did_repository,
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
        let didcomm_message = DidCommEncryptedService::generate(
            &self.did_repository,
            &my_did,
            &destination_did,
            &self.did_accessor.get_my_keyring(),
            &message,
            None,
            now,
            "",
        )
        .await
        .map_err(GenerateDidcommMessageUseCaseError::DidCommEncryptedServiceGenerateError)?;

        let result = serde_json::to_string(&didcomm_message)?;

        self.message_activity_repository
            .add_create_activity(CreatedMessageActivityRequest {
                message_id,
                from: my_did,
                to: destination_did,
                operation_tag,
                is_encrypted: true,
                occurred_at: now,
            })
            .await
            .map_err(GenerateDidcommMessageUseCaseError::MessageActivityHttpError)?;

        Ok(result)
    }

    pub async fn verify(
        &self,
        message: &str,
        now: DateTime<Utc>,
    ) -> Result<VerifiableCredentials, VerifyDidcommMessageUseCaseError<D::VerifyError, R::Error>>
    {
        let message = serde_json::from_str::<DidCommMessage>(message)?;
        let verified = DidCommEncryptedService::verify(
            &self.did_repository,
            &self.did_accessor.get_my_keyring(),
            &message,
        )
        .await
        .map_err(VerifyDidcommMessageUseCaseError::DidCommEncryptedServiceVerifyError)?;
        let verified = verified.message;
        let from_did = verified.issuer.id.clone();
        // check in verified. maybe exists?
        let my_did = self.did_accessor.get_my_did();
        let container = verified.clone().credential_subject.container;
        let message = serde_json::from_value::<EncodedMessage>(container)?;

        self.message_activity_repository
            .add_verify_activity(VerifiedMessageActivityRequest {
                from: from_did,
                to: my_did,
                message_id: message.message_id,
                verified_at: now,
                status: VerifiedStatus::Valid,
            })
            .await
            .map_err(VerifyDidcommMessageUseCaseError::MessageActivityHttpError)?;

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

    use nodex_didcomm::didcomm::encrypted::DidCommEncryptedServiceGenerateError;
    use nodex_didcomm::didcomm::encrypted::DidCommEncryptedServiceVerifyError;

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

        let usecase = DidcommMessageUseCase::new(
            MockMessageActivityRepository::verify_success(),
            repo,
            MockDidAccessor::new(presets.to_did, presets.to_keyring),
        );

        let verified = usecase.verify(&generated, Utc::now()).await.unwrap();
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

            if let Err(GenerateDidcommMessageUseCaseError::DidCommEncryptedServiceGenerateError(
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

            if let Err(GenerateDidcommMessageUseCaseError::MessageActivityHttpError(_)) = generated
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

            let usecase = DidcommMessageUseCase::new(
                MockMessageActivityRepository::verify_success(),
                MockDidRepository::empty(),
                MockDidAccessor::new(presets.to_did, presets.to_keyring),
            );

            let verified = usecase.verify(&generated, Utc::now()).await;

            if let Err(VerifyDidcommMessageUseCaseError::DidCommEncryptedServiceVerifyError(
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

            let usecase = DidcommMessageUseCase::new(
                MockMessageActivityRepository::verify_fail(),
                presets.create_mock_did_repository(),
                MockDidAccessor::new(presets.to_did, presets.to_keyring),
            );

            let verified = usecase.verify(&generated, Utc::now()).await;

            if let Err(VerifyDidcommMessageUseCaseError::MessageActivityHttpError(_)) = verified {
            } else {
                panic!("unexpected result: {:?}", verified);
            }
        }
    }
}
