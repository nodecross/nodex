use super::utils::milliseconds_to_time;
use crate::controllers::errors::AgentErrorCode;
use crate::repository::log_repository::LogStoreRepository;
use crate::services::studio::Studio;
use axum::extract::Json;
use axum::http::StatusCode;
use protocol::cbor::types::{Log, TimeValue};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    message: String,
    #[serde(default)]
    occurred_at: u64,
}

pub async fn handler(
    Json(MessageContainer {
        message,
        occurred_at,
    }): Json<MessageContainer>,
) -> Result<StatusCode, AgentErrorCode> {
    if message.is_empty() {
        return Err(AgentErrorCode::SendLogNoMessage);
    }
    let occurred_at =
        milliseconds_to_time(occurred_at).ok_or(AgentErrorCode::SendLogInvalidOccurredAt)?;
    let log = Log {
        message: TimeValue(occurred_at, message),
    };
    let studio = Studio::new();
    match LogStoreRepository::save(&studio, log).await {
        Ok(_) => {
            log::info!("save log");
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::SendEventInternal)?
        }
    }
}
