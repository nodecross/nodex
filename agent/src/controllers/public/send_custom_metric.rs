use super::utils::milliseconds_to_time;
use crate::{
    controllers::errors::AgentErrorCode,
    repository::custom_metric_repository::CustomMetricStoreRequest,
    usecase::custom_metric_usecase::CustomMetricUsecase,
};
use axum::extract::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    key: String,
    value: f32,
    #[serde(default)]
    occurred_at: u64,
}

pub async fn handler(Json(json): Json<MessageContainer>) -> Result<(), AgentErrorCode> {
    if json.key.is_empty() {
        Err(AgentErrorCode::SendCustomMetricNoKey)?
    }

    let occurred_at = milliseconds_to_time(json.occurred_at)
        .ok_or(AgentErrorCode::SendCustomMetricInvalidOccurredAt)?;

    let usecase = CustomMetricUsecase::new();
    match usecase
        .save(CustomMetricStoreRequest {
            key: json.key,
            value: json.value,
            occurred_at,
        })
        .await
    {
        Ok(_) => {
            log::info!("sent custom metrics");
            Ok(())
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::SendCustomMetricInternal)?
        }
    }
}
