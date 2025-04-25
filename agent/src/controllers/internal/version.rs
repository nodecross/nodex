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
