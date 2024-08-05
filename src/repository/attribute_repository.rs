use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct AttributeStoreRequest {
    pub key_name: String,
    pub value: String,
}

#[trait_variant::make(Send)]
pub trait AttributeStoreRepository {
    async fn save(&self, request: AttributeStoreRequest) -> anyhow::Result<()>;
}
