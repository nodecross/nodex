use super::utils;
use crate::controllers::errors::AgentErrorCode;
use crate::nodex::utils::did_accessor::DidAccessorImpl;
use crate::nodex::utils::webvh_client::DidWebvhDataStoreImpl;
use crate::server_config;
use crate::{
    services::studio::Studio,
    usecase::didcomm_message_usecase::{
        DidcommMessageUseCase, VerifyDidcommMessageUseCaseError as U,
    },
};
use axum::extract::Json;
use chrono::Utc;
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
    let now = Utc::now();
    let base_url = server_config()
        .map_err(|_| AgentErrorCode::VerifyDidcommMessageInternal)?
        .did_http_endpoint();
    let datasotre = DidWebvhDataStoreImpl::new(base_url);
    let webvh = DidWebvhServiceImpl::new(datasotre);

    let mut usecase = DidcommMessageUseCase::new(Studio::new(), webvh, DidAccessorImpl {});

    match serde_json::from_str::<DidCommMessage>(&json.message) {
        Err(e) => {
            log::warn!("json error: {}", e);
            Err(AgentErrorCode::VerifyDidcommMessageJsonError)?
        }
        Ok(message) => match usecase.verify(message, now).await {
            Ok(v) => Ok(Json(v)),
            Err(e) => match e {
                U::MessageActivity(e) => Err(utils::handle_status(e)),
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
