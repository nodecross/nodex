use crate::core::document::*;
use crate::core::interfaces::did_operation::*;
use crate::core::operator::*;
use crate::did_unid::did::*;
use crate::did_unid::interfaces::keyring_model::*;
use crate::did_unid::keyring::*;

#[derive(Debug)]
pub struct UNiDKlass {
  operator: UNiDDidOperator,
  context: Option<UNiDContextInternal>,
}

const SIGNING_KEY_ID: &str = "signingKey";

impl Default for UNiDKlass {
  fn default() -> Self {
    Self::new()
  }
}
impl UNiDKlass {
  pub fn new() -> Self {
    UNiDKlass {
      operator: UNiDDidOperator::new(None),
      context: None,
    }
  }

  pub fn get_context(&self) -> Option<UNiDContextInternal> {
    self.context.clone()
  }

  pub fn create_did(
    &self,
    r#type: KeyRingType,
    options: Option<MnemonicKeyringOptions>,
  ) -> UNiDDid {
    assert_eq!(r#type, KeyRingType::Mnemonic);
    let mut keyring = MnemonicKeyring::create_keyring(options);
    let purpose: Vec<String> = ["auth".to_string(), "general".to_string()].to_vec();
    let create_request: DIDCreateRequest = DIDCreateRequest {
      public_keys: [keyring
        .sign
        .to_public_key(SIGNING_KEY_ID.to_string(), purpose)]
      .to_vec(),
      commitment_keys: CommitmentKeys {
        recovery: keyring.recovery.to_jwk(),
        update: keyring.update.to_jwk(),
      },
      service_endpoints: [].to_vec(),
    };

    let document: UNiDDidDocument = self.operator.create(create_request);
    keyring.set_did(document.identifier());
    UNiDDid::new(UNiDDidContext {
      context: self.get_context(),
      keyring,
      operator: self.operator.clone(),
    })
  }

  pub fn get_did_document(&self, params: DIDResolutionRequest) -> UNiDDidDocument {
    self.operator.resolve(params)
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  fn it_should_create_did() {
    let unid_klass: UNiDKlass = Default::default();
    let unid_did: UNiDDid = unid_klass.create_did(KeyRingType::Mnemonic, None);
    assert!(!unid_did.get_identifier().is_empty());
  }

  #[test]
  fn it_should_create_did_and_resolve() {
    let unid_klass: UNiDKlass = Default::default();
    let unid_did: UNiDDid = unid_klass.create_did(KeyRingType::Mnemonic, None);
    let params: DIDResolutionRequest = DIDResolutionRequest {
      did: unid_did.get_identifier(),
    };
    let did_document: UNiDDidDocument = unid_klass.get_did_document(params);
    assert!(did_document
      .identifier()
      .contains(&unid_did.get_identifier()[..]));
    assert_eq!(&did_document.identifier()[..13], "did:unid:test")
  }
}
