use crate::nodex::utils::did_accessor::DidAccessor;
use crate::repository::message_activity_repository::*;
use chrono::DateTime;
use chrono::Utc;
use nodex_didcomm::{
    did::did_repository::DidRepository,
    verifiable_credentials::{did_vc::DidVcService, types::VerifiableCredentials},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub struct VerifiableMessageUseCase<R, D, A>
where
    R: MessageActivityRepository,
    D: DidRepository + DidVcService,
    A: DidAccessor,
{
    did_repository: D,
    message_activity_repository: R,
    did_accessor: A,
}

#[derive(Debug, Error)]
pub enum CreateVerifiableMessageUseCaseError<E, F>
where
    E: std::error::Error,
    F: std::error::Error,
{
    #[error("vc service error: {0}")]
    DidVcServiceGenerateError(E),
    #[error("message activity error: {0}")]
    MessageActivityHttpError(F),
    #[error("destination did not found")]
    DestinationNotFound,
    #[error("failed serialize/deserialize : {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum VerifyVerifiableMessageUseCaseError<E, F>
where
    E: std::error::Error,
    F: std::error::Error,
{
    #[error("vc service error: {0}")]
    DidVcServiceVerifyError(E),
    #[error("message activity error: {0}")]
    MessageActivityHttpError(F),
    #[error("This message is not addressed to me")]
    NotAddressedToMe,
    #[error("failed serialize/deserialize : {0}")]
    JsonError(#[from] serde_json::Error),
}

impl<R, D, A> VerifiableMessageUseCase<R, D, A>
where
    R: MessageActivityRepository,
    D: DidRepository + DidVcService,
    A: DidAccessor,
{
    pub fn new(message_activity_repository: R, did_repository: D, did_accessor: A) -> Self {
        VerifiableMessageUseCase {
            did_repository,
            message_activity_repository,
            did_accessor,
        }
    }
    pub async fn generate(
        &self,
        destination_did: String,
        message: String,
        operation_tag: String,
        now: DateTime<Utc>,
    ) -> Result<String, CreateVerifiableMessageUseCaseError<D::GenerateError, R::Error>> {
        self.did_repository
            .find_identifier(&destination_did)
            .await
            .ok()
            .and_then(|x| x)
            .ok_or(CreateVerifiableMessageUseCaseError::DestinationNotFound)?;

        let message_id = Uuid::new_v4();
        let my_did = self.did_accessor.get_my_did();
        let message = EncodedMessage {
            message_id,
            payload: message,
            destination_did: destination_did.clone(),
            created_at: now.to_rfc3339(),
        };

        let message = serde_json::to_value(message)?;
        let vc = DidVcService::generate(
            &self.did_repository,
            &my_did,
            &self.did_accessor.get_my_keyring(),
            &message,
            now,
        )
        .map_err(CreateVerifiableMessageUseCaseError::DidVcServiceGenerateError)?;

        let result = serde_json::to_string(&vc)?;

        self.message_activity_repository
            .add_create_activity(CreatedMessageActivityRequest {
                message_id,
                from: my_did,
                to: destination_did,
                operation_tag,
                is_encrypted: false,
                occurred_at: now,
            })
            .await
            .map_err(CreateVerifiableMessageUseCaseError::MessageActivityHttpError)?;
        Ok(result)
    }

    pub async fn verify(
        &self,
        message: &str,
        now: DateTime<Utc>,
    ) -> Result<VerifiableCredentials, VerifyVerifiableMessageUseCaseError<D::VerifyError, R::Error>>
    {
        let vc = serde_json::from_str::<VerifiableCredentials>(message)?;
        let vc = DidVcService::verify(&self.did_repository, vc)
            .await
            .map_err(VerifyVerifiableMessageUseCaseError::DidVcServiceVerifyError)?;
        let container = vc.clone().credential_subject.container;

        let message = serde_json::from_value::<EncodedMessage>(container)?;

        let from_did = vc.issuer.id.clone();
        let my_did = self.did_accessor.get_my_did();

        if message.destination_did != my_did {
            return Err(VerifyVerifiableMessageUseCaseError::NotAddressedToMe);
        }

        self.message_activity_repository
            .add_verify_activity(VerifiedMessageActivityRequest {
                from: from_did,
                to: my_did,
                message_id: message.message_id,
                verified_at: now,
                status: VerifiedStatus::Valid,
            })
            .await
            .map_err(VerifyVerifiableMessageUseCaseError::MessageActivityHttpError)?;
        Ok(vc)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct EncodedMessage {
    pub message_id: Uuid,
    pub payload: String,
    pub destination_did: String,
    pub created_at: String,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::nodex::utils::did_accessor::mocks::MockDidAccessor;
    use crate::repository::did_repository::mocks::MockDidRepository;

    use crate::usecase::test_util::*;
    use nodex_didcomm::verifiable_credentials::did_vc::DidVcServiceVerifyError;
    use serde_json::Value;
    use tests::mocks::MockMessageActivityRepository;

    #[tokio::test]
    async fn test_create_and_verify() {
        let presets = TestPresets::default();
        let repository = presets.create_mock_did_repository();

        let usecase = VerifiableMessageUseCase::new(
            MockMessageActivityRepository::create_success(),
            repository.clone(),
            MockDidAccessor::new(presets.from_did, presets.from_keyring.clone()),
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

        let result: Value = serde_json::from_str(&generated).unwrap();

        let message_id = result["credentialSubject"]["container"]["message_id"]
            .as_str()
            .unwrap();

        assert_eq!(
            result["credentialSubject"]["container"],
            serde_json::json!({
                "message_id": message_id,
                "payload": "Hello",
                "destination_did": &presets.to_did,
                "created_at": now.to_rfc3339(),
            })
        );

        let usecase = VerifiableMessageUseCase::new(
            MockMessageActivityRepository::verify_success(),
            repository.clone(),
            MockDidAccessor::new(presets.to_did, presets.from_keyring),
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

            let usecase = VerifiableMessageUseCase::new(
                MockMessageActivityRepository::create_success(),
                MockDidRepository::empty(),
                MockDidAccessor::new(presets.from_did, presets.from_keyring),
            );

            let message = "Hello".to_string();

            let now = Utc::now();
            let generated = usecase
                .generate(presets.to_did, message, "test".to_string(), now)
                .await;

            if let Err(CreateVerifiableMessageUseCaseError::DestinationNotFound) = generated {
            } else {
                panic!("unexpected result: {:?}", generated);
            }
        }

        #[tokio::test]
        async fn test_generate_add_activity_failed() {
            let presets = TestPresets::default();
            let repository = presets.create_mock_did_repository();

            let usecase = VerifiableMessageUseCase::new(
                MockMessageActivityRepository::create_fail(),
                repository.clone(),
                MockDidAccessor::new(presets.from_did, presets.from_keyring),
            );

            let message = "Hello".to_string();

            let now = Utc::now();
            let generated = usecase
                .generate(presets.to_did, message, "test".to_string(), now)
                .await;

            if let Err(CreateVerifiableMessageUseCaseError::MessageActivityHttpError(_)) = generated
            {
            } else {
                panic!("unexpected result: {:?}", generated);
            }
        }
    }

    mod verify_failed {
        use crate::nodex::utils::did_accessor::mocks::MockDidAccessor;

        use super::*;

        async fn create_test_message_for_verify_test(presets: TestPresets) -> String {
            let repository = presets.create_mock_did_repository();

            let usecase = VerifiableMessageUseCase::new(
                MockMessageActivityRepository::create_success(),
                repository.clone(),
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

            let result: Value = serde_json::from_str(&generated).unwrap();

            let message_id = result["credentialSubject"]["container"]["message_id"]
                .as_str()
                .unwrap();

            assert_eq!(
                result["credentialSubject"]["container"],
                serde_json::json!({
                    "message_id": message_id,
                    "payload": "Hello",
                    "destination_did": &presets.to_did,
                    "created_at": now.to_rfc3339(),
                })
            );

            generated
        }

        #[tokio::test]
        async fn test_verify_not_addressed_to_me() {
            let presets = TestPresets::default();
            let repository = presets.create_mock_did_repository();

            let generated = create_test_message_for_verify_test(presets.clone()).await;

            let _message = "Hello".to_string();

            let usecase = VerifiableMessageUseCase::new(
                MockMessageActivityRepository::verify_success(),
                repository.clone(),
                MockDidAccessor::new("wrong_did".to_owned(), presets.from_keyring),
            );

            let verified = usecase.verify(&generated, Utc::now()).await;

            if let Err(VerifyVerifiableMessageUseCaseError::NotAddressedToMe) = verified {
            } else {
                panic!("unexpected result: {:?}", verified);
            }
        }

        #[tokio::test]
        async fn test_verify_did_not_found() {
            let presets = TestPresets::default();

            let repository = MockDidRepository::empty();

            let usecase = VerifiableMessageUseCase::new(
                MockMessageActivityRepository::verify_success(),
                repository.clone(),
                MockDidAccessor::new(presets.clone().to_did, presets.clone().to_keyring),
            );

            let generated = create_test_message_for_verify_test(presets).await;
            let verified = usecase.verify(&generated, Utc::now()).await;

            if let Err(VerifyVerifiableMessageUseCaseError::DidVcServiceVerifyError(
                DidVcServiceVerifyError::DidDocNotFound(_),
            )) = verified
            {
            } else {
                panic!("unexpected result: {:?}", verified);
            }
        }

        #[tokio::test]
        async fn test_verify_add_activity_failed() {
            let presets = TestPresets::default();
            let repository = presets.create_mock_did_repository();

            let generated = create_test_message_for_verify_test(presets.clone()).await;

            let usecase = VerifiableMessageUseCase::new(
                MockMessageActivityRepository::verify_fail(),
                repository.clone(),
                MockDidAccessor::new(presets.to_did, presets.to_keyring),
            );

            let verified = usecase.verify(&generated, Utc::now()).await;

            if let Err(VerifyVerifiableMessageUseCaseError::MessageActivityHttpError(_)) = verified
            {
            } else {
                panic!("unexpected result: {:?}", verified);
            }
        }
    }
}
