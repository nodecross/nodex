use crate::{controllers::errors::AgentErrorCode, services::nodex::NodeX};
use axum::extract::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// NOTE: POST /internal/version
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: Value,
}

pub async fn handler_get() -> Json<Value> {
    Json(serde_json::json!({ "version": env!("CARGO_PKG_VERSION")}))
}

pub async fn handler_update(
    Json(json): Json<MessageContainer>,
) -> Result<Json<&'static str>, AgentErrorCode> {
    let binary_url = match json.message["binary_url"].as_str() {
        Some(url) => url,
        None => Err(AgentErrorCode::VersionNoBinaryUrl)?,
    };
    let nodex = NodeX::new();
    match nodex.update_version(binary_url).await {
        Ok(_) => Ok(Json("ok")),
        Err(e) => {
            log::error!("{}", e);
            Err(AgentErrorCode::VersionInternal)?
        },
    }
}
