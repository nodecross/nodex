use protocol::cbor::types::Event;

#[trait_variant::make(Send)]
pub trait EventStoreRepository {
    async fn save(&self, request: Vec<Event>) -> anyhow::Result<()>;
}
