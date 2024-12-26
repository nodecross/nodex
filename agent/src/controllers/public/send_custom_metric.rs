use super::utils::milliseconds_to_time;
use crate::{
    controllers::errors::AgentErrorCode,
    repository::custom_metric_repository::CustomMetricStoreRequest,
    usecase::custom_metric_usecase::CustomMetricUsecase,
};
use axum::extract::Json;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    key: String,
    value: f32,
    #[serde(default)]
    occurred_at: u64,
}

pub async fn handler(
    Json(json): Json<Vec<MessageContainer>>,
) -> Result<StatusCode, AgentErrorCode> {
    let metrics = json
        .iter()
        .map(|m| {
            if m.key.is_empty() {
                return Err(AgentErrorCode::SendCustomMetricNoKey);
            }

            let occurred_at = milliseconds_to_time(m.occurred_at)
                .ok_or(AgentErrorCode::SendCustomMetricInvalidOccurredAt)?;

            Ok(CustomMetricStoreRequest {
                key: m.key.clone(),
                value: m.value,
                occurred_at,
            })
        })
        .collect::<Result<Vec<CustomMetricStoreRequest>, AgentErrorCode>>()?;

    let usecase = CustomMetricUsecase::new();
    match usecase.save(metrics).await {
        Ok(_) => {
            log::info!("sent custom metrics");
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::SendCustomMetricInternal)?
        }
    }
}
