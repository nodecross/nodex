use super::utils::milliseconds_to_time;
use crate::{
    controllers::errors::AgentErrorCode, repository::event_repository::EventStoreRequest,
    usecase::event_usecase::EventUsecase,
};
use axum::extract::Json;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    key: String,
    #[serde(default)]
    detail: String,
    #[serde(default)]
    occurred_at: u64,
}

pub async fn handler(Json(json): Json<MessageContainer>) -> Result<StatusCode, AgentErrorCode> {
    if json.key.is_empty() {
        Err(AgentErrorCode::SendEventNoKey)?
    }
    if json.detail.is_empty() {
        Err(AgentErrorCode::SendEventNoDetail)?
    }

    let occurred_at =
        milliseconds_to_time(json.occurred_at).ok_or(AgentErrorCode::SendEventInvalidOccurredAt)?;

    let usecase = EventUsecase::new();
    match usecase
        .save(EventStoreRequest {
            key: json.key,
            detail: json.detail,
            occurred_at,
        })
        .await
    {
        Ok(_) => {
            log::info!("save event");
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::SendEventInternal)?
        }
    }
}
