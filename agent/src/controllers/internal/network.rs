use crate::{controllers::errors::AgentErrorCode, services::studio::Studio};
use axum::extract::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// NOTE: POST /internal/version
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: Value,
}

pub async fn handler(
    Json(_): Json<MessageContainer>,
) -> Result<Json<&'static str>, AgentErrorCode> {
    let mut studio = Studio::new();
    match studio.network().await {
        Ok(_) => Ok(Json("ok")),
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::NetworkInternal)?
        }
    }
}
