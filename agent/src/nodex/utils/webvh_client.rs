use protocol::did_webvh::domain::did_document::DidDocument;
use protocol::did_webvh::domain::did_log_entry::DidLogEntry;
use protocol::did_webvh::infra::did_webvh_data_store::DidWebvhDataStore;
use url::Url;

#[allow(dead_code)]
#[derive(Clone)]
pub struct DidWebvhDataStoreImpl {
    base_url: Url,
    client: reqwest::Client,
    use_https: bool,
}

impl DidWebvhDataStoreImpl {
    pub fn new(base_url: Url) -> Self {
        let use_https = base_url.scheme() == "https";
        Self {
            base_url,
            client: reqwest::Client::new(),
            use_https,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DidWebvhDataStoreImplError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("Error from server: code={code:?} message={message:?}")]
    ServerError {
        code: reqwest::StatusCode,
        message: String,
    },
}

#[allow(unused_variables)]
impl DidWebvhDataStore for DidWebvhDataStoreImpl {
    type Error = DidWebvhDataStoreImplError;

    async fn create(
        &mut self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<DidDocument, Self::Error> {
        let scheme = if self.use_https { "https" } else { "http" };
        let response = self
            .client
            .post(format!("{}://{}", scheme, did_path))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(did_log_entries)?)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(DidWebvhDataStoreImplError::ServerError {
                code: status,
                message: response.text().await?,
            });
        }
        Ok(response.json().await?)
    }

    async fn get(&mut self, did_path: &str) -> Result<Vec<DidLogEntry>, Self::Error> {
        let scheme = if self.use_https { "https" } else { "http" };
        let response = self
            .client
            .get(format!("{}://{}", scheme, did_path))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(DidWebvhDataStoreImplError::ServerError {
                code: status,
                message: response.text().await?,
            });
        }
        Ok(response.json().await?)
    }

    async fn update(
        &mut self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<DidDocument, Self::Error> {
        let scheme = if self.use_https { "https" } else { "http" };
        let response = self
            .client
            .put(format!("{}://{}", scheme, did_path))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(did_log_entries)?)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(DidWebvhDataStoreImplError::ServerError {
                code: status,
                message: response.text().await?,
            });
        }
        Ok(response.json().await?)
    }
    async fn deactivate(
        &mut self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<DidDocument, Self::Error> {
        let scheme = if self.use_https { "https" } else { "http" };
        let response = self
            .client
            .delete(format!("{}://{}", scheme, did_path))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(did_log_entries)?)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(DidWebvhDataStoreImplError::ServerError {
                code: status,
                message: response.text().await?,
            });
        }
        Ok(response.json().await?)
    }
}
