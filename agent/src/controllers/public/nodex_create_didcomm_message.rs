use super::utils;
use crate::controllers::errors::AgentErrorCode;
use crate::nodex::utils::did_accessor::DidAccessorImpl;
use crate::nodex::utils::webvh_client::DidWebvhDataStoreImpl;
use crate::server_config;
use crate::usecase::didcomm_message_usecase::GenerateDidcommMessageUseCaseError as U;
use crate::{services::studio::Studio, usecase::didcomm_message_usecase::DidcommMessageUseCase};
use axum::extract::Json;
use chrono::Utc;
use protocol::did_webvh::service::service_impl::DidWebvhServiceImpl;
use serde::{Deserialize, Serialize};
use url::Url;

// NOTE: POST /create-didcomm-message
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    destination_did: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    operation_tag: String,
}

pub async fn handler(Json(json): Json<MessageContainer>) -> Result<String, AgentErrorCode> {
    if json.destination_did.is_empty() {
        Err(AgentErrorCode::CreateDidCommMessageNoDestinationDid)?
    }
    if json.message.is_empty() {
        Err(AgentErrorCode::CreateDidCommMessageNoMessage)?
    }
    if json.operation_tag.is_empty() {
        Err(AgentErrorCode::CreateDidCommMessageNoOperationTag)?
    }

    let base_url = {
        let base_url = &server_config().did_http_endpoint();
        Url::parse(base_url).expect("failed to parse url")
    };
    let datasotre = DidWebvhDataStoreImpl::new(base_url);
    let webvh = DidWebvhServiceImpl::new(datasotre);

    let now = Utc::now();

    let mut usecase = DidcommMessageUseCase::new(Studio::new(), webvh, DidAccessorImpl {});

    match usecase
        .generate(json.destination_did, json.message, json.operation_tag, now)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => match e {
            U::MessageActivity(e) => Err(utils::handle_status(e)),
            U::Json(e) => {
                log::warn!("json error: {}", e);
                Err(AgentErrorCode::CreateDidcommMessageInternal)?
            }
            U::Generate(e) => {
                log::warn!("didcomm generate error: {}", e);
                Err(AgentErrorCode::CreateDidcommMessageInternal)?
            }
            U::InvalidDid(e) => {
                log::warn!("invalid did error: {}", e);
                Err(AgentErrorCode::CreateDidcommMessageInternal)?
            }
        },
    }
}
