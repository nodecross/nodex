use crate::nodex::utils::did_accessor::DidAccessor;
use crate::repository::message_activity_repository::*;
use chrono::DateTime;
use chrono::Utc;
use protocol::{
    did::did_repository::DidRepository,
    verifiable_credentials::{did_vc::DidVcService, types::VerifiableCredentials},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub struct VerifiableMessageUseCase<R, D, S, A>
where
    R: MessageActivityRepository,
    D: DidRepository,
    S: DidVcService,
    A: DidAccessor,
{
    did_repository: D,
    vc_service: S,
    message_activity_repository: R,
    did_accessor: A,
}

#[derive(Debug, Error)]
pub enum CreateVerifiableMessageUseCaseError<D, E, F>
where
    D: std::error::Error,
    E: std::error::Error,
    F: std::error::Error,
{
    #[error("vc service error: {0}")]
    DidVcServiceGenerate(E),
    #[error("message activity error: {0}")]
    MessageActivity(F),
    #[error("destination did not found")]
    DestinationNotFound(Option<D>),
    #[error("failed serialize/deserialize : {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum VerifyVerifiableMessageUseCaseError<E, F>
where
    E: std::error::Error,
    F: std::error::Error,
{
    #[error("vc service error: {0}")]
    DidVcServiceVerify(E),
    #[error("message activity error: {0}")]
    MessageActivity(F),
    #[error("This message is not addressed to me")]
    NotAddressedToMe,
    #[error("failed serialize/deserialize : {0}")]
    Json(#[from] serde_json::Error),
}

impl<R, D, S, A> VerifiableMessageUseCase<R, D, S, A>
where
    R: MessageActivityRepository,
    D: DidRepository,
    S: DidVcService,
    A: DidAccessor,
{
    pub fn new(
        message_activity_repository: R,
        vc_service: S,
        did_accessor: A,
        did_repository: D,
    ) -> Self {
        VerifiableMessageUseCase {
            did_repository,
            vc_service,
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
    ) -> Result<
        String,
        CreateVerifiableMessageUseCaseError<D::FindIdentifierError, S::GenerateError, R::Error>,
    > {
        use CreateVerifiableMessageUseCaseError::DestinationNotFound;
        match self.did_repository.find_identifier(&destination_did).await {
            Err(e) => Err(DestinationNotFound(Some(e))),
            Ok(None) => Err(DestinationNotFound(None)),
            Ok(Some(_)) => Ok(()),
        }?;

        let message_id = Uuid::new_v4();
        let my_did = self.did_accessor.get_my_did();
        let message = EncodedMessage {
            message_id,
            payload: message,
            destination_did: destination_did.clone(),
            created_at: now.to_rfc3339(),
        };

        let message = serde_json::to_value(message)?;
        let model = VerifiableCredentials::new(my_did.clone(), message, now);
        let vc = self
            .vc_service
            .generate(model, &self.did_accessor.get_my_keyring())
            .map_err(CreateVerifiableMessageUseCaseError::DidVcServiceGenerate)?;

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
            .map_err(CreateVerifiableMessageUseCaseError::MessageActivity)?;
        Ok(result)
    }

    pub async fn verify(
        &self,
        message: VerifiableCredentials,
        now: DateTime<Utc>,
    ) -> Result<VerifiableCredentials, VerifyVerifiableMessageUseCaseError<S::VerifyError, R::Error>>
    {
        let vc = self
            .vc_service
            .verify(message)
            .await
            .map_err(VerifyVerifiableMessageUseCaseError::DidVcServiceVerify)?;
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
            .map_err(VerifyVerifiableMessageUseCaseError::MessageActivity)?;
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
    use protocol::verifiable_credentials::did_vc::DidVcServiceVerifyError;
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
            repository.clone(),
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
            repository.clone(),
        );

        let generated = serde_json::from_str::<VerifiableCredentials>(&generated).unwrap();
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

            let usecase = VerifiableMessageUseCase::new(
                MockMessageActivityRepository::create_success(),
                MockDidRepository::empty(),
                MockDidAccessor::new(presets.from_did, presets.from_keyring),
                MockDidRepository::empty(),
            );

            let message = "Hello".to_string();

            let now = Utc::now();
            let generated = usecase
                .generate(presets.to_did, message, "test".to_string(), now)
                .await;

            if let Err(CreateVerifiableMessageUseCaseError::DestinationNotFound(_)) = generated {
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
                repository.clone(),
            );

            let message = "Hello".to_string();

            let now = Utc::now();
            let generated = usecase
                .generate(presets.to_did, message, "test".to_string(), now)
                .await;

            if let Err(CreateVerifiableMessageUseCaseError::MessageActivity(_)) = generated {
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
                repository.clone(),
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
                repository.clone(),
            );

            let generated = serde_json::from_str::<VerifiableCredentials>(&generated).unwrap();
            let verified = usecase.verify(generated, Utc::now()).await;

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
                repository.clone(),
            );

            let generated = create_test_message_for_verify_test(presets).await;
            let generated = serde_json::from_str::<VerifiableCredentials>(&generated).unwrap();
            let verified = usecase.verify(generated, Utc::now()).await;

            if let Err(VerifyVerifiableMessageUseCaseError::DidVcServiceVerify(
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
                repository.clone(),
            );

            let generated = serde_json::from_str::<VerifiableCredentials>(&generated).unwrap();
            let verified = usecase.verify(generated, Utc::now()).await;

            if let Err(VerifyVerifiableMessageUseCaseError::MessageActivity(_)) = verified {
            } else {
                panic!("unexpected result: {:?}", verified);
            }
        }
    }
}
