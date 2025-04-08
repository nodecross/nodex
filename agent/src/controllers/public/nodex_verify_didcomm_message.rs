use crate::controllers::errors::AgentErrorCode;
use crate::nodex::utils::did_accessor::DidAccessorImpl;
use crate::nodex::utils::webvh_client::DidWebvhDataStoreImpl;
use crate::server_config;
use crate::usecase::didcomm_message_usecase::{
    DidcommMessageUseCase, VerifyDidcommMessageUseCaseError as U,
};
use axum::extract::Json;
use protocol::did_webvh::service::service_impl::DidWebvhServiceImpl;
use protocol::didcomm::types::DidCommMessage;
use serde::{Deserialize, Serialize};

// NOTE: POST /verify-verifiable-message
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    message: String,
}

pub async fn handler(Json(json): Json<MessageContainer>) -> Result<Json<String>, AgentErrorCode> {
    let base_url = server_config()
        .map_err(|_| AgentErrorCode::VerifyDidcommMessageInternal)?
        .did_http_endpoint();
    let datasotre = DidWebvhDataStoreImpl::new(base_url);
    let webvh = DidWebvhServiceImpl::new(datasotre);

    let mut usecase = DidcommMessageUseCase::new(webvh, DidAccessorImpl {});

    match serde_json::from_str::<DidCommMessage>(&json.message) {
        Err(e) => {
            log::warn!("json error: {}", e);
            Err(AgentErrorCode::VerifyDidcommMessageJsonError)?
        }
        Ok(message) => match usecase.verify(message).await {
            Ok(v) => Ok(Json(v)),
            Err(e) => match e {
                U::NotAddressedToMe => {
                    log::warn!("message not addressed to me: {}", e);
                    Err(AgentErrorCode::VerifyDidcommMessageNotAddressedToMe)?
                }
                U::FindSender(e) => {
                    log::warn!("cannot find sender: {}", e);
                    Err(AgentErrorCode::VerifyDidcommMessageNoSender)?
                }
                U::Verify(e) => {
                    log::warn!("Failed to verify didcomm message: {}", e);
                    Err(AgentErrorCode::VerifyDidcommMessageInternal)?
                }
                U::Json(e) => {
                    log::warn!("json error: {}", e);
                    Err(AgentErrorCode::VerifyDidcommMessageInternal)?
                }
            },
        },
    }
}
