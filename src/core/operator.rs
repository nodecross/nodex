use super::super::utils::http_client::*;
use super::document::*;
use super::interfaces::{did_document::*, did_operation::*};
use super::payload::*;
use std::collections::HashMap;

pub type KV = HashMap<String, String>;

pub struct UNiDDidResolverContext {
  pub _debug: Option<bool>,
  pub _endpoint: Option<String>,
}

pub struct UNiDDidOperator {
  pub debug: bool,
  pub endpoint: String,
  pub client: HttpClient,
}

impl UNiDDidOperator {
  pub fn new(_context: Option<UNiDDidResolverContext>) -> Self {
    let endpoint: String;
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
    let client_config: AxiosRequestConfig = AxiosRequestConfig {
      base_url: endpoint.to_string(),
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
  pub fn resolve(&self, params: DIDResolutionRequest) -> UNiDDidDocument {
    let client = &self.client;
    let resolve_directory = "/api/v1/identifiers".to_string();
    let request_path: String = format!("{}/{}", resolve_directory, params.did);
    let res_string: String = client.get_text(Some(request_path));
    assert!(serde_json::from_str::<serde_json::Value>(&res_string).is_ok());
    let did_root: Root = serde_json::from_str(&res_string).unwrap();
    UNiDDidDocument::new(did_root.did_document)
  }
  pub fn resolve_str(&self, params: DIDResolutionRequest) -> String {
    let client = &self.client;
    let resolve_directory = "/api/v1/identifiers".to_string();
    let request_path: String = format!("{}/{}", resolve_directory, params.did);
    let res_string: String = client.get_text(Some(request_path));
    res_string
  }

  pub fn create(&self, params: DIDCreateRequest) -> UNiDDidDocument {
    let payload = Payload::did_create_payload(params);
    let client = &self.client;
    let resolve_directory = "/api/v1/operations".to_string();
    let mut payload_map = HashMap::new();
    payload_map.insert("create".to_string(), payload.type_field);
    payload_map.insert("delta".to_string(), payload.delta);
    payload_map.insert("suffix_data".to_string(), payload.suffix_data);
    let res_string: String = client.post_text(payload_map, Some(resolve_directory));
    assert!(serde_json::from_str::<serde_json::Value>(&res_string).is_ok());
    let did_root: Root = serde_json::from_str(&res_string).unwrap();
    UNiDDidDocument::new(did_root.did_document)
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  fn it_should_did_document_text_nonempty() {
    let resolver_context: UNiDDidResolverContext = UNiDDidResolverContext {
      _debug: Some(false),
      _endpoint: Some("https://did.getunid.io".to_string()),
    };
    let did_operator: UNiDDidOperator = UNiDDidOperator::new(Some(resolver_context));
    let params: DIDResolutionRequest = DIDResolutionRequest {
      did: "did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw".to_string(),
    };
    let res: String = did_operator.resolve_str(params);
    assert!(!res.is_empty());
  }

  #[test]
  fn it_should_resolve_did_document() {
    let did: String = "did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw".to_string();
    let op: UNiDDidOperator = UNiDDidOperator::new(None);
    let res: UNiDDidDocument = op.resolve(DIDResolutionRequest {
      did: did.to_string(),
    });
    assert!(res.identifier().contains(&did[..]));
    assert_eq!(&res.identifier()[..13], "did:unid:test")
  }

  #[test]
  fn it_should_create_did_document() {
    let did: String = "did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw".to_string();
    let op: UNiDDidOperator = UNiDDidOperator::new(None);
    let res: UNiDDidDocument = op.resolve(DIDResolutionRequest {
      did: did.to_string(),
    });
    assert!(res.identifier().contains(&did[..]));
    assert_eq!(&res.identifier()[..13], "did:unid:test")
  }
}
