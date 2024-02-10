use chrono::{DateTime, Utc};
use serde::Serialize;
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
    pub message_id: Uuid,
    pub verified_at: DateTime<Utc>,
    pub status: VerifiedStatus,
}

#[derive(Clone, Debug, Serialize)]
pub enum VerifiedStatus {
    Valid,
    Invalid,
}

#[async_trait::async_trait]
pub trait MessageActivityRepository {
    async fn add_create_activity(
        &self,
        request: CreatedMessageActivityRequest,
    ) -> anyhow::Result<()>;
    async fn add_verify_activity(
        &self,
        request: VerifiedMessageActivityRequest,
    ) -> anyhow::Result<()>;
}
