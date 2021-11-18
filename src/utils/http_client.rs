use crate::core::interfaces::did_operation::KV;

pub struct HttpClientContext {
    pub debug: bool,
}

pub struct AxiosRequestConfig {
    pub base_url: String,
}

#[derive(Clone, Debug)]
pub struct HttpClient {
    pub base_url: String,
    pub instance: reqwest::blocking::Client,
}

impl HttpClient {
    pub fn new(_config: Option<AxiosRequestConfig>, _context: Option<HttpClientContext>) -> Self {
        let base_url: String;

        if _config.is_some() {
            base_url = _config.as_ref().unwrap().base_url.to_string();
        } else {
            base_url = "https://did.getunid.io".to_string();
        }

        let client: reqwest::blocking::Client = reqwest::blocking::Client::new();

        HttpClient {
            instance: client,
            base_url,
        }
    }

    pub fn get(&self, _request_path: Option<String>) -> reqwest::blocking::Response {
        let request_url: String;

        if _request_path.is_some() {
            request_url = format!("{}{}", self.base_url, _request_path.unwrap());
        } else {
            request_url = self.base_url.to_string();
        }

        let res = self.instance.get(request_url).send();

        assert!(res.is_ok());

        res.unwrap()
    }

    pub fn get_kv(&self, _request_path: Option<String>) -> KV {
        let res_ok = HttpClient::get(self, _request_path);
        let res_ok_json = res_ok.json::<KV>();

        assert!(res_ok_json.is_ok());

        res_ok_json.unwrap()
    }

    pub fn get_text(&self, _request_path: Option<String>) -> String {
        let res_ok = HttpClient::get(self, _request_path);

        res_ok.text().unwrap()
    }

    pub fn get_serde(&self, _request_path: Option<String>) -> serde_json::Value {
        let res_ok_text = HttpClient::get_text(self, _request_path);

        let res_json: serde_json::Value = serde_json::from_str(&res_ok_text).unwrap();

        res_json
    }

    pub fn post(&self, payload: KV, _request_path: Option<String>) -> reqwest::blocking::Response {
        let request_url: String;

        if _request_path.is_some() {
            request_url = format!("{}{}", self.base_url, _request_path.unwrap());
        } else {
            request_url = self.base_url.to_string();
        }

        let res = self.instance.post(request_url).json(&payload).send();

        assert!(res.is_ok());

        res.unwrap()
    }

    pub fn post_text(&self, payload: KV, _request_path: Option<String>) -> String {
        let res_ok = HttpClient::post(self, payload, _request_path);

        res_ok.text().unwrap()
    }

    pub fn post_serde(&self, payload: KV, _request_path: Option<String>) -> serde_json::Value {
        let res_ok_text = HttpClient::post_text(self, payload, _request_path);

        let res_json: serde_json::Value = serde_json::from_str(&res_ok_text).unwrap();

        res_json
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn it_should_post_hashmap_correct() {
        let client_config: AxiosRequestConfig = AxiosRequestConfig {
            base_url: "http://httpbin.org/post".to_string(),
        };

        let client_context: HttpClientContext = HttpClientContext { debug: false };
        let client = HttpClient::new(Some(client_config), Some(client_context));
        let mut payload_map = std::collections::HashMap::new();
        payload_map.insert("key".to_string(), "value".to_string());
        let res: serde_json::Value = client.post_serde(payload_map, None);

        assert!(!res.is_null());
        assert!(!res["json"].is_null());
        assert!(res["json"]["key"].is_string());
        assert_eq!(res["json"]["key"], "value");
    }

    #[test]
    fn it_should_get_endpoint_correct() {
        let client_config: AxiosRequestConfig = AxiosRequestConfig {
            base_url: "https://httpbin.org/ip".to_string(),
        };

        let client_context: HttpClientContext = HttpClientContext { debug: false };
        let client = HttpClient::new(Some(client_config), Some(client_context));

        assert_eq!(client.base_url, "https://httpbin.org/ip".to_string());
    }

    #[test]
    fn it_should_get_key_origin_and_value() {
        let client_config: AxiosRequestConfig = AxiosRequestConfig {
            base_url: "https://httpbin.org/ip".to_string(),
        };

        let client_context: HttpClientContext = HttpClientContext { debug: false };
        let client = HttpClient::new(Some(client_config), Some(client_context));
        let res_kv: KV = client.get_kv(None);

        assert!(!res_kv.is_empty());
        assert!(res_kv.contains_key("origin"));
        assert!(res_kv.get("origin").is_some());
        assert!(!res_kv.get("origin").unwrap().is_empty());
    }

    #[test]
    fn it_should_did_document_serde_nonempty() {
        let client_config: AxiosRequestConfig = AxiosRequestConfig {
            base_url: "https://did.getunid.io".to_string(),
        };

        let client_context: HttpClientContext = HttpClientContext { debug: false };
        let client = HttpClient::new(Some(client_config), Some(client_context));
        let res: serde_json::Value = client.get_serde(Some(
            "/api/v1/identifiers/did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw"
                .to_string(),
        ));

        assert!(!res.is_null());
        assert!(res["@context"].is_string());
        assert!(res["didDocument"]["id"].is_string());
        assert_eq!(res["didDocument"]["id"],"did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw:eyJkZWx0YV9oYXNoIjoiRWlCSVgwcVRiYi1jWFF5OEhFcFJXMm83dUJ3dF90Ym53bHZTSDVpU0hUcjJBQSIsInJlY292ZXJ5X2NvbW1pdG1lbnQiOiJFaUE1a0JYaWhobjNIWXEtRUQ0czFEVTE2cHpWOFZfZ2ZfSVBDaDQ2VW9Uc3BRIn0.eyJwYXRjaGVzIjpbeyJhY3Rpb24iOiJyZXBsYWNlIiwiZG9jdW1lbnQiOnsicHVibGljX2tleXMiOlt7ImlkIjoic2lnbmluZ0tleSIsInR5cGUiOiJFY2RzYVNlY3AyNTZrMVZlcmlmaWNhdGlvbktleTIwMTkiLCJqd2siOnsia3R5IjoiRUMiLCJjcnYiOiJzZWNwMjU2azEiLCJ4IjoiUFNLQWszMTNpSEVNbDE3WGdaYl91LTBJS3FiX0pRb2c4akk0WXgzelBBRSIsInkiOiIwckFtbXg5YU5tVTg0c2QzakxxM2VKTzc4YUM1OFJrSnJRbUx4U2xwems4In0sInB1cnBvc2UiOlsiYXV0aCIsImdlbmVyYWwiXX1dLCJzZXJ2aWNlX2VuZHBvaW50cyI6W119fV0sInVwZGF0ZV9jb21taXRtZW50IjoiRWlDckhSZXRLcks3V0NMdmtjVFc5ZDFRVlJiRzB2cEZIRi1rdktDb2NSdjdCQSJ9");
    }
}
