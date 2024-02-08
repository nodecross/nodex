use crate::nodex::sidetree::payload::DIDResolutionResponse;
// use thiserror::Error;

#[async_trait::async_trait]
pub trait DidRepository {
    async fn create_identifier(&self) -> anyhow::Result<DIDResolutionResponse>;
    async fn find_identifier(&self, did: &str) -> anyhow::Result<Option<DIDResolutionResponse>>;
}

// #[derive(Debug, Error)]
// pub enum DidRepositoryError {
//     KeyPairingError,
//     FindIdentifierError,
// }
