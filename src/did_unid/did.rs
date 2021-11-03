use crate::core::operator::*;
use crate::did_unid::interfaces::keyring_model::*;

#[derive(Clone, Debug)]
pub enum UNiDNetworkType {
  Mainnet,
  Testnet,
}

#[derive(Clone, Debug)]
pub struct UNiDContextExternal {
  pub client_id: String,
  pub client_secret: String,
  pub env_network: Option<UNiDNetworkType>,
}

#[derive(Clone, Debug)]
pub struct UNiDContextInternal {
  pub client_id: String,
  pub client_secret: String,
  pub env_network: Option<UNiDNetworkType>,
}

#[derive(Clone, Debug)]
pub struct UNiDDidContext {
  pub context: Option<UNiDContextInternal>,
  pub keyring: MnemonicKeyring,
  pub operator: UNiDDidOperator,
}

#[derive(Debug)]
pub struct UNiDDid {
  pub context: Option<UNiDContextInternal>,
  pub keyring: MnemonicKeyring,
  pub operator: UNiDDidOperator,
}

impl UNiDDid {
  pub fn new(context: UNiDDidContext) -> Self {
    UNiDDid {
      keyring: context.keyring,
      operator: context.operator,
      context: context.context,
    }
  }
  pub fn get_identifier(&self) -> String {
    self.keyring.get_identifier(None)
  }
}
