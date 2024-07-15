use anyhow::Context;
use async_trait::async_trait;
use nodex_didcomm::did::sidetree::client::{SidetreeHttpClient, SidetreeHttpClientResponse};
use url::{ParseError, Url};

#[derive(Clone)]
pub struct SideTreeClient {
    base_url: Url,
    client: reqwest::Client,
}

impl SideTreeClient {
    pub fn new(base_url: &str) -> anyhow::Result<Self> {
        let base_url =
            Url::parse(base_url).context("NODEX_DID_HTTP_ENDPOINT must be a valid URL")?;
        Ok(Self {
            base_url,
            client: reqwest::Client::new(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SideTreeClientError {
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[async_trait]
impl SidetreeHttpClient for SideTreeClient {
    type Error = SideTreeClientError;
    async fn post_create_identifier(
        &self,
        body: &str,
    ) -> Result<SidetreeHttpClientResponse, Self::Error> {
        let url = self.base_url.join("/api/v1/operations")?;

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        let response = SidetreeHttpClientResponse::new(status, body);

        Ok(response)
    }
    async fn get_find_identifier(
        &self,
        did: &str,
    ) -> Result<SidetreeHttpClientResponse, Self::Error> {
        let url = self
            .base_url
            .join(&format!("/api/v1/identifiers/{}", did))?;

        let response = self.client.get(url).send().await?;

        let response = SidetreeHttpClientResponse::new(response.status(), response.text().await?);

        Ok(response)
    }
}
