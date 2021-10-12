use super::super::utils::http_client::*;
use std::collections::HashMap;

pub type KV = HashMap<String, String>;

pub struct UNiDDidResolverContext {
  pub _debug: Option<bool>,
  pub _endpoint: Option<String>,
  pub _path: Option<String>,
}

pub struct UNiDDidOperator {
  pub debug: bool,
  pub endpoint: String,
  pub client: HttpClient,
}

impl UNiDDidOperator {
  pub fn new(_context: Option<UNiDDidResolverContext>) -> Self {
    let endpoint: String;
    let path: String;
    let debug: bool;
    if _context.is_some() && _context.as_ref().unwrap()._debug.is_some() {
      debug = *_context.as_ref().unwrap()._debug.as_ref().unwrap();
    } else {
      debug = false;
    };
    if _context.is_some()
      && _context.as_ref().unwrap()._endpoint.is_some()
      && !_context
        .as_ref()
        .unwrap()
        ._endpoint
        .as_ref()
        .unwrap()
        .is_empty()
    {
      endpoint = _context
        .as_ref()
        .unwrap()
        ._endpoint
        .as_ref()
        .unwrap()
        .to_string();
    } else {
      endpoint = "https://did.getunid.io".to_string();
    }
    if _context.is_some() && _context.as_ref().unwrap()._path.is_some() {
      path = _context
        .as_ref()
        .unwrap()
        ._path
        .as_ref()
        .unwrap()
        .to_string();
    } else {
      path = "/api/v1/identifiers/".to_string();
    }
    let client_config: AxiosRequestConfig = AxiosRequestConfig {
      base_url: endpoint.to_string(),
      path,
    };
    let client_context: HttpClientContext = HttpClientContext { debug };
    let client = HttpClient::new(Some(client_config), Some(client_context));
    let unid_did_operator: UNiDDidOperator = UNiDDidOperator {
      endpoint,
      client,
      debug,
    };
    unid_did_operator
  }
  // pub fn resolve(params: DIDResolutionRequest) -> bool {}
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  fn it_should_get_key_origin_and_value_nonempty() {
    let resolver_context: UNiDDidResolverContext = UNiDDidResolverContext {
      _path: Some("".to_string()),
      _debug: Some(true),
      _endpoint: Some("https://httpbin.org/ip".to_string()),
    };
    let did_operator: UNiDDidOperator = UNiDDidOperator::new(Some(resolver_context));
    let http_client: HttpClient = did_operator.client;
    let res: KV = http_client.get_kv(None);
    assert!(res.contains_key("origin"));
    assert!(res.get("origin").is_some());
    assert!(! res.get("origin").unwrap().is_empty());
  }
  #[test]
  fn it_should_get_debug_false() {
    let resolver_context: UNiDDidResolverContext = UNiDDidResolverContext {
      _path: Some("".to_string()),
      _debug: Some(false),
      _endpoint: Some("https://httpbin.org/ip".to_string()),
    };
    let did: UNiDDidOperator = UNiDDidOperator::new(Some(resolver_context));
    assert!(! did.debug);
  }

  #[test]
  fn it_should_get_debug_true() {
    let resolver_context: UNiDDidResolverContext = UNiDDidResolverContext {
      _path: Some("".to_string()),
      _debug: Some(true),
      _endpoint: Some("https://httpbin.org/ip".to_string()),
    };
    let did: UNiDDidOperator = UNiDDidOperator::new(Some(resolver_context));
    assert!(did.debug);
  }

  #[test]
  fn it_should_get_endpoint_nonempty() {
    let resolver_context: UNiDDidResolverContext = UNiDDidResolverContext {
      _path: Some("".to_string()),
      _debug: Some(true),
      _endpoint: Some("https://httpbin.org/ip".to_string()),
    };
    let did: UNiDDidOperator = UNiDDidOperator::new(Some(resolver_context));
    assert!(! did.endpoint.is_empty());
  }
  #[test]
  fn it_should_did_document_text_nonempty() {
    let resolver_context: UNiDDidResolverContext = UNiDDidResolverContext {
      _path: Some("/api/v1/identifiers/".to_string()),
      _debug: Some(false),
      _endpoint: Some("https://did.getunid.io".to_string()),
    };
    let did_operator: UNiDDidOperator = UNiDDidOperator::new(Some(resolver_context));
    let http_client: HttpClient = did_operator.client;
    let params: DIDResolutionRequest = DIDResolutionRequest {
      did: "did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw".to_string(),
    };
    let res: String = http_client.get_text(Some(params));
    assert!(! res.is_empty());
  }
  #[test]
  fn it_should_did_document_serde_nonempty() {
    let resolver_context: UNiDDidResolverContext = UNiDDidResolverContext {
      _path: Some("/api/v1/identifiers/".to_string()),
      _debug: Some(false),
      _endpoint: Some("https://did.getunid.io".to_string()),
    };
    let did_operator: UNiDDidOperator = UNiDDidOperator::new(Some(resolver_context));
    let http_client: HttpClient = did_operator.client;
    let params: DIDResolutionRequest = DIDResolutionRequest {
      did: "did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw".to_string(),
    };
    let res: serde_json::Value = http_client.get_serde(Some(params));
    assert!(! res.is_null());
    assert!(res["didDocument"]["id"].is_string());
    assert_eq!(res["didDocument"]["id"],"did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw:eyJkZWx0YV9oYXNoIjoiRWlCSVgwcVRiYi1jWFF5OEhFcFJXMm83dUJ3dF90Ym53bHZTSDVpU0hUcjJBQSIsInJlY292ZXJ5X2NvbW1pdG1lbnQiOiJFaUE1a0JYaWhobjNIWXEtRUQ0czFEVTE2cHpWOFZfZ2ZfSVBDaDQ2VW9Uc3BRIn0.eyJwYXRjaGVzIjpbeyJhY3Rpb24iOiJyZXBsYWNlIiwiZG9jdW1lbnQiOnsicHVibGljX2tleXMiOlt7ImlkIjoic2lnbmluZ0tleSIsInR5cGUiOiJFY2RzYVNlY3AyNTZrMVZlcmlmaWNhdGlvbktleTIwMTkiLCJqd2siOnsia3R5IjoiRUMiLCJjcnYiOiJzZWNwMjU2azEiLCJ4IjoiUFNLQWszMTNpSEVNbDE3WGdaYl91LTBJS3FiX0pRb2c4akk0WXgzelBBRSIsInkiOiIwckFtbXg5YU5tVTg0c2QzakxxM2VKTzc4YUM1OFJrSnJRbUx4U2xwems4In0sInB1cnBvc2UiOlsiYXV0aCIsImdlbmVyYWwiXX1dLCJzZXJ2aWNlX2VuZHBvaW50cyI6W119fV0sInVwZGF0ZV9jb21taXRtZW50IjoiRWlDckhSZXRLcks3V0NMdmtjVFc5ZDFRVlJiRzB2cEZIRi1rdktDb2NSdjdCQSJ9");
  }
  #[test]
  fn it_should_did_document_serde_did_correct() {
    let resolver_context: UNiDDidResolverContext = UNiDDidResolverContext {
      _path: Some("/api/v1/identifiers/".to_string()),
      _debug: Some(false),
      _endpoint: Some("https://did.getunid.io".to_string()),
    };
    let did_operator: UNiDDidOperator = UNiDDidOperator::new(Some(resolver_context));
    let http_client: HttpClient = did_operator.client;
    let params: DIDResolutionRequest = DIDResolutionRequest {
      did: "did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw".to_string(),
    };
    let res: serde_json::Value = http_client.get_serde(Some(params));
    assert!(res["didDocument"]["id"].is_string());
    assert_eq!(res["didDocument"]["id"],"did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw:eyJkZWx0YV9oYXNoIjoiRWlCSVgwcVRiYi1jWFF5OEhFcFJXMm83dUJ3dF90Ym53bHZTSDVpU0hUcjJBQSIsInJlY292ZXJ5X2NvbW1pdG1lbnQiOiJFaUE1a0JYaWhobjNIWXEtRUQ0czFEVTE2cHpWOFZfZ2ZfSVBDaDQ2VW9Uc3BRIn0.eyJwYXRjaGVzIjpbeyJhY3Rpb24iOiJyZXBsYWNlIiwiZG9jdW1lbnQiOnsicHVibGljX2tleXMiOlt7ImlkIjoic2lnbmluZ0tleSIsInR5cGUiOiJFY2RzYVNlY3AyNTZrMVZlcmlmaWNhdGlvbktleTIwMTkiLCJqd2siOnsia3R5IjoiRUMiLCJjcnYiOiJzZWNwMjU2azEiLCJ4IjoiUFNLQWszMTNpSEVNbDE3WGdaYl91LTBJS3FiX0pRb2c4akk0WXgzelBBRSIsInkiOiIwckFtbXg5YU5tVTg0c2QzakxxM2VKTzc4YUM1OFJrSnJRbUx4U2xwems4In0sInB1cnBvc2UiOlsiYXV0aCIsImdlbmVyYWwiXX1dLCJzZXJ2aWNlX2VuZHBvaW50cyI6W119fV0sInVwZGF0ZV9jb21taXRtZW50IjoiRWlDckhSZXRLcks3V0NMdmtjVFc5ZDFRVlJiRzB2cEZIRi1rdktDb2NSdjdCQSJ9");
  }
}
