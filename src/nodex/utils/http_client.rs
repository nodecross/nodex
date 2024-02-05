use reqwest::{
    header::{HeaderMap, HeaderValue},
    Url,
};

pub struct HttpClientConfig {
    pub base_url: String,
}

#[derive(Clone, Debug)]
pub struct HttpClient {
    pub base_url: Url,
    pub instance: reqwest::Client,
}

impl HttpClient {
    pub fn new(_config: &HttpClientConfig) -> anyhow::Result<Self> {
        let url = Url::parse(&_config.base_url.to_string())?;
        let client: reqwest::Client = reqwest::Client::new();

        Ok(HttpClient {
            instance: client,
            base_url: url,
        })
    }

    fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers
    }

    pub async fn get(&self, _path: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(_path);

        let response = self
            .instance
            .get(&url.unwrap().to_string())
            .headers(self.default_headers())
            .send()
            .await?;

        Ok(response)
    }

    pub async fn post(&self, _path: &str, body: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(_path);

        let response = self
            .instance
            .post(&url.unwrap().to_string())
            .headers(self.default_headers())
            .body(body.to_string())
            .send()
            .await?;

        Ok(response)
    }

    #[allow(dead_code)]
    pub async fn put(&self, _path: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(_path);

        let response = self
            .instance
            .put(&url.unwrap().to_string())
            .headers(self.default_headers())
            .send()
            .await?;

        Ok(response)
    }

    #[allow(dead_code)]
    pub async fn delete(&self, _path: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(_path);

        let response = self
            .instance
            .delete(&url.unwrap().to_string())
            .headers(self.default_headers())
            .send()
            .await?;

        Ok(response)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Res {
        origin: String,
    }

    #[actix_rt::test]
    #[ignore]
    async fn it_should_success_get() {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let res = match client.get("/get").await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let json: Res = match res.json().await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(!json.origin.is_empty());
    }

    #[actix_rt::test]
    #[ignore]
    async fn it_should_success_post() {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let res = match client.post("/post", r#"{"key":"value"}"#).await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let json: Res = match res.json().await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(!json.origin.is_empty());
    }

    #[actix_rt::test]
    #[ignore]
    async fn it_should_success_put() {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let res = match client.put("/put").await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let json: Res = match res.json().await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(!json.origin.is_empty());
    }

    #[actix_rt::test]
    #[ignore]
    async fn it_should_success_delete() {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let res = match client.delete("/delete").await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let json: Res = match res.json().await {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(!json.origin.is_empty());
    }
}
