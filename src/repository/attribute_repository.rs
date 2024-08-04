use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct AttributeStoreRequest {
    pub key_name: String,
    pub value: String,
}

#[async_trait::async_trait]
pub trait AttributeStoreRepository {
    async fn save(&self, request: AttributeStoreRequest) -> anyhow::Result<()>;
}
