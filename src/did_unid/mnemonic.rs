use super::interfaces::keyring_model::*;
use super::secp256k1::*;
use crate::runtime::bip32::*;

const BASE_DERIVATION_PATH: &str = "m/44\'/0\'/0\'/0";
const SIGN_DERIVATION_PATH: &str = "m/44\'/0\'/0\'/0/10";
const UPDATE_DERIVATION_PATH: &str = "m/44\'/0\'/0\'/0/20";
const RECOVERY_DERIVATION_PATH: &str = "m/44\'/0\'/0\'/0/30";
const ENCRYPTION_DERIVATION_PATH: &str = "m/44\'/0\'/0\'/0/40";

impl MnemonicKeyring {
  pub fn create_keyring(options: Option<MnemonicKeyringOptions>) -> Self {
    let context = MnemonicKeyring::generate_bip39_seed(options);
    let sign =
      MnemonicKeyring::generate_secp256k1(context.clone(), SIGN_DERIVATION_PATH.to_string());
    let update =
      MnemonicKeyring::generate_secp256k1(context.clone(), UPDATE_DERIVATION_PATH.to_string());
    let recovery =
      MnemonicKeyring::generate_secp256k1(context.clone(), RECOVERY_DERIVATION_PATH.to_string());
    let encrypt =
      MnemonicKeyring::generate_secp256k1(context.clone(), ENCRYPTION_DERIVATION_PATH.to_string());
    let mut instance: MnemonicKeyring = MnemonicKeyring::new(MnemonicKeyringContext {
      context,
      sign,
      update,
      recovery,
      encrypt,
    });
    let model = instance.save_context(None, None);

    instance.set_keyring_model(model);
    instance
  }

  pub fn new(context: MnemonicKeyringContext) -> Self {
    MnemonicKeyring {
      base_derivation_path: BASE_DERIVATION_PATH.to_string(),
      sign_derivation_path: SIGN_DERIVATION_PATH.to_string(),
      update_derivation_path: UPDATE_DERIVATION_PATH.to_string(),
      recovery_derivation_path: RECOVERY_DERIVATION_PATH.to_string(),
      encryption_derivation_path: ENCRYPTION_DERIVATION_PATH.to_string(),
      context: context.context,
      sign: context.sign,
      update: context.update,
      recovery: context.recovery,
      encrypt: context.encrypt,
      model: None,
    }
  }

  pub fn generate_bip39_seed(options: Option<MnemonicKeyringOptions>) -> BIP39Context {
    let keyring_options: MnemonicKeyringOptions = match options {
      Some(v) => v,
      None => MnemonicKeyringOptions { length: 24 },
    };

    let mnemonic: String = crate::runtime::bip39::BIP39::generate_mnemonic(keyring_options.length);
    let seed: Vec<u8> = crate::runtime::bip39::BIP39::mnemonic_to_seed(mnemonic.clone(), None);

    BIP39Context {
      mnemonic: Some(mnemonic),
      seed,
    }
  }
  pub fn generate_node_by_derivation_path(
    context: BIP39Context,
    derivation_path: String,
  ) -> BIP32Interface {
    BIP32::get_node(context.seed, derivation_path)
  }
  pub fn generate_secp256k1(context: BIP39Context, derivation_path: String) -> Secp256k1 {
    let node: BIP32Interface =
      MnemonicKeyring::generate_node_by_derivation_path(context, derivation_path);

    Secp256k1::new(Secp256k1Context {
      public: node.public_key,
      private: node.private_key,
    })
  }
  pub fn set_did(&mut self, did: String) {
    let model = self.save_context(Some(did), None);
    self.set_keyring_model(model);
  }

  pub fn get_identifier(&self, key_id: Option<String>) -> String {
    assert!(self.model.is_some());
    assert!(self.model.clone().unwrap().did.is_some());
    if key_id.is_none() {
      self.model.clone().unwrap().did.unwrap()
    } else {
      format!(
        "{}#{}",
        self.model.clone().unwrap().did.unwrap(),
        key_id.unwrap()
      )
    }
  }
  fn save_context(
    &self,
    did: Option<String>,
    options: Option<SaveContextOptions>,
  ) -> MnemonicKeyringModel {
    let clone = self.clone();
    let mut mnemonic: Option<String> = clone.context.mnemonic;

    if options.is_some()
      && options.as_ref().unwrap().remove_mnemonic.is_some()
      && options.as_ref().unwrap().remove_mnemonic.unwrap()
    {
      mnemonic = None;
    };

    MnemonicKeyringModel {
      did,
      seed: clone.context.seed,
      sign: clone.sign,
      update: clone.update,
      recovery: clone.recovery,
      encrypt: clone.encrypt,
      mnemonic,
    }
  }
  pub fn set_keyring_model(&mut self, model: MnemonicKeyringModel) {
    self.model = Some(model);
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  fn it_should_generate_bip39_seed() {
    let bip39: BIP39Context = MnemonicKeyring::generate_bip39_seed(None);
    assert_eq!(bip39.seed.len(), 64);
    assert!(bip39.mnemonic.is_some());
  }

  #[test]
  fn it_should_generate_node_by_derivation_path() {
    let bip39_context: BIP39Context = MnemonicKeyring::generate_bip39_seed(None);
    let bip32_interface: BIP32Interface = MnemonicKeyring::generate_node_by_derivation_path(
      bip39_context,
      BASE_DERIVATION_PATH.to_string(),
    );
    assert_eq!(bip32_interface.private_key.len(), 32);
    assert_eq!(bip32_interface.public_key.len(), 33);
  }

  #[test]
  fn it_should_generate_secp256k1() {
    let bip39_context: BIP39Context = MnemonicKeyring::generate_bip39_seed(None);
    let secp256k1: Secp256k1 =
      MnemonicKeyring::generate_secp256k1(bip39_context, BASE_DERIVATION_PATH.to_string());
    assert_eq!(secp256k1._private.len(), 32);
    assert_eq!(secp256k1._public.len(), 65);
  }

  #[test]
  fn it_should_create_keyring() {
    let keyring: MnemonicKeyring = MnemonicKeyring::create_keyring(None);
    assert!(keyring.model.is_some());
  }
}
