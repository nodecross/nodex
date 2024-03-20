use chrono::{DateTime, Utc};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct CreatedMessageActivityRequest {
    pub message_id: Uuid,
    pub from: String,
    pub to: String,
    pub operation_tag: String,
    pub is_encrypted: bool,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct VerifiedMessageActivityRequest {
    pub from: String,
    pub to: String,
    pub message_id: Uuid,
    pub verified_at: DateTime<Utc>,
    pub status: VerifiedStatus,
}

#[derive(Clone, Debug, Serialize)]
pub enum VerifiedStatus {
    Valid,
    Invalid,
}

#[derive(Error, Debug)]
pub enum MessageActivityHttpError {
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait MessageActivityRepository {
    async fn add_create_activity(
        &self,
        request: CreatedMessageActivityRequest,
    ) -> Result<(), MessageActivityHttpError>;
    async fn add_verify_activity(
        &self,
        request: VerifiedMessageActivityRequest,
    ) -> anyhow::Result<()>;
}
