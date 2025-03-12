use super::utils::milliseconds_to_time;
use crate::{controllers::errors::AgentErrorCode, usecase::event_usecase::EventUsecase};
use axum::extract::Json;
use axum::http::StatusCode;
use protocol::cbor::types::{Event, TimeValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    key: String,
    #[serde(default)]
    detail: String,
    #[serde(default)]
    occurred_at: u64,
}

pub async fn handler(
    Json(json): Json<Vec<MessageContainer>>,
) -> Result<StatusCode, AgentErrorCode> {
    let events = json
        .into_iter()
        .try_fold(
            HashMap::new(),
            |mut acc: HashMap<String, Vec<TimeValue<String>>>,
             MessageContainer {
                 key,
                 detail,
                 occurred_at,
             }| {
                if key.is_empty() {
                    return Err(AgentErrorCode::SendEventNoKey);
                }
                let occurred_at = milliseconds_to_time(occurred_at)
                    .ok_or(AgentErrorCode::SendEventInvalidOccurredAt)?;
                acc.entry(key)
                    .or_default()
                    .push(TimeValue(occurred_at, detail));
                Ok(acc)
            },
        )?
        .into_iter()
        .map(|(key, details)| Event { key, details })
        .collect();

    let usecase = EventUsecase::new();
    match usecase.save(events).await {
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
