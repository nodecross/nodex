use http::StatusCode;

#[derive(Clone, Debug)]
pub struct SidetreeHttpClientResponse {
    pub(crate) status_code: StatusCode,
    pub(crate) body: String,
}

impl SidetreeHttpClientResponse {
    pub fn new(status_code: StatusCode, body: String) -> Self {
        Self { status_code, body }
    }
}

#[trait_variant::make(Send)]
pub trait SidetreeHttpClient {
    type Error: std::error::Error;
    async fn post_create_identifier(
        &self,
        body: &str,
    ) -> Result<SidetreeHttpClientResponse, Self::Error>;
    async fn get_find_identifier(
        &self,
        did: &str,
    ) -> Result<SidetreeHttpClientResponse, Self::Error>;
}
