use protocol::cbor::types::Log;

#[trait_variant::make(Send)]
pub trait LogStoreRepository {
    async fn save(&self, request: Log) -> anyhow::Result<()>;
}
