use std::collections::HashMap;

type KV = HashMap<String, String>;

pub struct HttpClientContext {
  pub debug: bool,
}

pub struct AxiosRequestConfig {
  pub base_url: String,
  pub path: String,
}

pub struct DIDResolutionRequest {
  pub did: String,
}

pub struct HttpClient {
  pub base_url: String,
  pub path: String,
  pub instance: reqwest::blocking::Client,
}

impl HttpClient {
  pub fn new(_config: Option<AxiosRequestConfig>, _context: Option<HttpClientContext>) -> Self {
    let base_url: String;
    let path: String;
    if _config.is_some() {
      base_url = _config.as_ref().unwrap().base_url.to_string();
      path = _config.as_ref().unwrap().path.to_string();
    } else {
      base_url = "https://did.getunid.io".to_string();
      path = "/api/v1/identifiers/".to_string();
    }
    let client: reqwest::blocking::Client = reqwest::blocking::Client::new();

    HttpClient {
      instance: client,
      base_url,
      path,
    }
  }

  pub fn get(&self, _params: Option<DIDResolutionRequest>) -> reqwest::blocking::Response {
    assert!(reqwest::Url::parse(&self.base_url).is_ok());
    let request_base: reqwest::Url = reqwest::Url::parse(&self.base_url).unwrap();
    assert!(request_base.join(&self.path).is_ok());
    let request_path: reqwest::Url = request_base.join(&self.path).unwrap();
    let params: DIDResolutionRequest;
    if let Some(..) = _params {
      params = _params.unwrap();
    } else {
      params = DIDResolutionRequest {
        did: "".to_string(),
      }
    }
    let request_path_str: &str = request_path.as_str();
    let request_url: String = format!("{}{}", request_path_str, &params.did);

    let res = self.instance.get(request_url).send();
    assert!(res.is_ok());

    res.unwrap()
  }

  pub fn get_kv(&self, _params: Option<DIDResolutionRequest>) -> KV {
    let res_ok = HttpClient::get(self, _params);
    let res_ok_json = res_ok.json::<KV>();
    assert!(res_ok_json.is_ok());
    res_ok_json.unwrap()
  }

  pub fn get_text(&self, _params: Option<DIDResolutionRequest>) -> String {
    let res_ok = HttpClient::get(self, _params);
    res_ok.text().unwrap()
  }

  pub fn get_serde(&self, _params: Option<DIDResolutionRequest>) -> serde_json::Value {
    let res_ok_text = HttpClient::get_text(self, _params);

    let res_json: serde_json::Value = serde_json::from_str(&res_ok_text).unwrap();

    res_json
  }
}
