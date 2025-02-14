use super::utils;
use crate::controllers::errors::AgentErrorCode;
use crate::nodex::utils::did_accessor::DidAccessorImpl;
use crate::usecase::verifiable_message_usecase::VerifyVerifiableMessageUseCaseError as U;
use crate::{
    services::studio::Studio, usecase::verifiable_message_usecase::VerifiableMessageUseCase,
};
use axum::extract::Json;
use chrono::Utc;
use protocol::verifiable_credentials::did_vc::DidVcServiceVerifyError as S;
use protocol::verifiable_credentials::types::VerifiableCredentials;
use serde::{Deserialize, Serialize};

// NOTE: POST /verify-verifiable-message
#[derive(Clone, Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    message: String,
}

pub async fn handler(
    Json(json): Json<MessageContainer>,
) -> Result<Json<VerifiableCredentials>, AgentErrorCode> {
    let now = Utc::now();

    let repo = utils::did_repository();
    let usecase =
        VerifiableMessageUseCase::new(Studio::new(), repo.clone(), DidAccessorImpl {}, repo);

    match serde_json::from_str::<VerifiableCredentials>(&json.message) {
        Err(e) => {
            log::warn!("json error: {}", e);
            Err(AgentErrorCode::VerifyVerifiableMessageJsonError)?
        }
        Ok(vc) => match usecase.verify(vc, now).await {
            Ok(v) => Ok(Json(v)),
            Err(e) => match e {
                U::MessageActivity(e) => Err(utils::handle_status(e)),
                U::DidVcServiceVerify(S::VerifyFailed(e)) => {
                    log::warn!("verify failed: {}", e);
                    Err(AgentErrorCode::VerifyVerifiableMessageVerifyFailed)?
                }
                U::DidVcServiceVerify(S::FindIdentifier(e)) => {
                    log::warn!("find identifier error: {}", e);
                    Err(AgentErrorCode::VerifyVerifiableMessageNoIdentifier)?
                }
                U::DidVcServiceVerify(S::DidDocNotFound(target)) => {
                    log::warn!("target DID not found. DID = {}", target);
                    Err(AgentErrorCode::VerifyVerifiableMessageNoTargetDid)?
                }
                U::NotAddressedToMe => {
                    log::warn!("this message is not addressed to me: {}", e);
                    Err(AgentErrorCode::VerifyVerifiableMessageNotAddressedToMe)?
                }
                U::Json(e) => {
                    log::warn!("json error: {}", e);
                    Err(AgentErrorCode::VerifyVerifiableMessageInternal)?
                }
                U::DidVcServiceVerify(S::PublicKeyNotFound(e)) => {
                    log::warn!("cannot find public key: {}", e);
                    Err(AgentErrorCode::VerifyVerifiableMessageNoPublicKey)?
                }
            },
        },
    }
}
