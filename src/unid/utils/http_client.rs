use reqwest::{Url, header::{HeaderMap, HeaderValue}};
use crate::unid::errors::UNiDError;

pub struct HttpClientConfig {
    pub base_url: String,
}

#[derive(Clone, Debug)]
pub struct HttpClient {
    pub base_url: Url,
    pub instance: reqwest::blocking::Client,
}

impl HttpClient {
    pub fn new(_config: &HttpClientConfig) -> Result<Self, UNiDError> {
        let url = match Url::parse(&_config.base_url.to_string()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let client: reqwest::blocking::Client = reqwest::blocking::Client::new();

        Ok(
            HttpClient {
                instance: client,
                base_url: url,
            }
        )
    }

    fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub fn get(&self, _path: &String) -> Result<reqwest::blocking::Response, UNiDError> {
        let url = self.base_url.join(&_path);

        match self.instance
            .get(&url.unwrap().to_string())
            .headers(self.default_headers())
            .send() {
                Ok(v) => Ok(v),
                Err(_) => Err(UNiDError{})
            }
    }

    pub fn post(&self, _path: &String) -> Result<reqwest::blocking::Response, UNiDError> {
        let url = self.base_url.join(&_path);

        match self.instance
            .post(&url.unwrap().to_string())
            .headers(self.default_headers())
            .send() {
                Ok(v) => Ok(v),
                Err(_) => Err(UNiDError{})
            }
    }

    pub fn put(&self, _path: &String) -> Result<reqwest::blocking::Response, UNiDError> {
        let url = self.base_url.join(&_path);

        match self.instance
            .put(&url.unwrap().to_string())
            .headers(self.default_headers())
            .send() {
                Ok(v) => Ok(v),
                Err(_) => Err(UNiDError{})
            }
    }

    pub fn delete(&self, _path: &String) -> Result<reqwest::blocking::Response, UNiDError> {
        let url = self.base_url.join(&_path);

        match self.instance
            .delete(&url.unwrap().to_string())
            .headers(self.default_headers())
            .send() {
                Ok(v) => Ok(v),
                Err(_) => Err(UNiDError{})
            }
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

    #[test]
    fn it_should_success_get() {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let res = match client.get(&("/get".to_string())) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let json: Res = match res.json() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(true, 0 < json.origin.len());
    }

    #[test]
    fn it_should_success_post() {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let res = match client.post(&("/post".to_string())) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let json: Res = match res.json() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(true, 0 < json.origin.len());
    }

    #[test]
    fn it_should_success_put() {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let res = match client.put(&("/put".to_string())) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let json: Res = match res.json() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(true, 0 < json.origin.len());
    }

    #[test]
    fn it_should_success_delete() {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://httpbin.org".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let res = match client.delete(&("/delete".to_string())) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let json: Res = match res.json() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(true, 0 < json.origin.len());
    }
}