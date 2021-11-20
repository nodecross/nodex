use serde::{Deserialize, Serialize};
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "HexKeyPair")]
#[serde(rename_all = "camelCase")]
pub struct Secp256k1 {
    pub _public: Vec<u8>,
    pub _private: Vec<u8>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MnemonicKeyringModel {
    pub did: Option<String>,
    pub seed: Vec<u8>,
    pub mnemonic: Option<String>,
    pub sign: Secp256k1,
    pub update: Secp256k1,
    pub recovery: Secp256k1,
    pub encrypt: Secp256k1,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BIP39Context {
    pub seed: Vec<u8>,
    pub mnemonic: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MnemonicKeyringContext {
    pub context: BIP39Context,
    pub sign: Secp256k1,
    pub update: Secp256k1,
    pub recovery: Secp256k1,
    pub encrypt: Secp256k1,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MnemonicKeyring {
    pub base_derivation_path: String,
    pub sign_derivation_path: String,
    pub update_derivation_path: String,
    pub recovery_derivation_path: String,
    pub encryption_derivation_path: String,
    pub context: BIP39Context,
    pub sign: Secp256k1,
    pub update: Secp256k1,
    pub recovery: Secp256k1,
    pub encrypt: Secp256k1,
    pub model: Option<MnemonicKeyringModel>,
}

pub type BIP39PhraseSize = usize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MnemonicKeyringOptions {
    pub length: BIP39PhraseSize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveContextOptions {
    pub remove_mnemonic: Option<bool>,
}
