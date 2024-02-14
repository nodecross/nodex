use super::secp256k1::{Secp256k1, Secp256k1Context, Secp256k1Error};
use crate::{
    app_config,
    config::{KeyPair, SingletonAppConfig},
    nodex::{
        extension::secure_keystore::{SecureKeyStore, SecureKeyStoreType},
        extension::{secure_keystore::SecureKeyStoreError, trng::Trng},
        runtime,
    },
};

use thiserror::Error;

pub struct KeyPairing {
    sign: Secp256k1,
    update: Secp256k1,
    recovery: Secp256k1,
    encrypt: Secp256k1,
    config: Box<SingletonAppConfig>,
    secure_keystore: SecureKeyStore,
}

#[derive(Error, Debug)]
pub enum KeyPairingError {
    #[error("secure keystore error")]
    SecureKeyStoreError(#[from] SecureKeyStoreError),
    #[error("key not found")]
    KeyNotFound,
    #[error("secp256k1 error")]
    KeyInitializationError(#[from] Secp256k1Error),
    #[error("Trng error")]
    TrngError(#[from] crate::nodex::extension::trng::TrngError),
    #[error("BIP32 error")]
    BIP32Error(#[from] runtime::bip32::BIP32Error),
    #[error("DID not found")]
    DIDNotFound,
}

impl KeyPairing {
    const SIGN_DERIVATION_PATH: &'static str = "m/44'/0'/0'/0/10";
    const UPDATE_DERIVATION_PATH: &'static str = "m/44'/0'/0'/0/20";
    const RECOVERY_DERIVATION_PATH: &'static str = "m/44'/0'/0'/0/30";
    const ENCRYPT_DERIVATION_PATH: &'static str = "m/44'/0'/0'/0/40";

    pub fn load_keyring() -> Result<Self, KeyPairingError> {
        let config = app_config();
        let secure_keystore = SecureKeyStore::new();

        fn load_secp256k1(
            secure_keystore: &SecureKeyStore,
            key_type: SecureKeyStoreType,
        ) -> Result<Secp256k1, KeyPairingError> {
            let key_pair = secure_keystore
                .read(&key_type)?
                .ok_or(KeyPairingError::KeyNotFound)?;

            Secp256k1::new(&Secp256k1Context {
                public: key_pair.public_key,
                secret: key_pair.secret_key,
            })
            .map_err(KeyPairingError::KeyInitializationError)
        }

        let sign = load_secp256k1(&secure_keystore, SecureKeyStoreType::Sign)?;
        let update = load_secp256k1(&secure_keystore, SecureKeyStoreType::Update)?;
        let recovery = load_secp256k1(&secure_keystore, SecureKeyStoreType::Recover)?;
        let encrypt = load_secp256k1(&secure_keystore, SecureKeyStoreType::Encrypt)?;

        Ok(KeyPairing {
            sign,
            update,
            recovery,
            encrypt,
            config,
            secure_keystore,
        })
    }

    pub fn create_keyring() -> Result<Self, KeyPairingError> {
        let config = app_config();
        let secure_keystore = SecureKeyStore::new();

        let trng = Trng::new();

        let seed = trng.read(&(256 / 8))?;

        let sign = Self::generate_secp256k1(&seed, Self::SIGN_DERIVATION_PATH)?;
        let update = Self::generate_secp256k1(&seed, Self::UPDATE_DERIVATION_PATH)?;
        let recovery = Self::generate_secp256k1(&seed, Self::RECOVERY_DERIVATION_PATH)?;
        let encrypt = Self::generate_secp256k1(&seed, Self::ENCRYPT_DERIVATION_PATH)?;

        Ok(KeyPairing {
            sign,
            update,
            recovery,
            encrypt,
            config,
            secure_keystore,
        })
    }

    pub fn get_sign_key_pair(&self) -> Secp256k1 {
        self.sign.clone()
    }

    pub fn get_update_key_pair(&self) -> Secp256k1 {
        self.update.clone()
    }

    pub fn get_recovery_key_pair(&self) -> Secp256k1 {
        self.recovery.clone()
    }

    pub fn get_encrypt_key_pair(&self) -> Secp256k1 {
        self.encrypt.clone()
    }

    pub fn generate_secp256k1(
        seed: &[u8],
        derivation_path: &str,
    ) -> Result<Secp256k1, KeyPairingError> {
        let node = runtime::bip32::BIP32::get_node(seed, derivation_path)?;

        Secp256k1::new(&Secp256k1Context {
            public: node.public_key,
            secret: node.private_key,
        })
        .map_err(KeyPairingError::KeyInitializationError)
    }

    pub fn save(&mut self, did: &str) {
        match self.secure_keystore.write(
            &SecureKeyStoreType::Sign,
            &KeyPair {
                public_key: self.get_sign_key_pair().get_public_key(),
                secret_key: self.get_sign_key_pair().get_secret_key(),
            },
        ) {
            Ok(_) => (),
            _ => panic!(),
        };
        match self.secure_keystore.write(
            &SecureKeyStoreType::Update,
            &KeyPair {
                public_key: self.get_update_key_pair().get_public_key(),
                secret_key: self.get_update_key_pair().get_secret_key(),
            },
        ) {
            Ok(_) => (),
            _ => panic!(),
        };
        match self.secure_keystore.write(
            &SecureKeyStoreType::Recover,
            &KeyPair {
                public_key: self.get_recovery_key_pair().get_public_key(),
                secret_key: self.get_recovery_key_pair().get_secret_key(),
            },
        ) {
            Ok(_) => (),
            _ => panic!(),
        };
        match self.secure_keystore.write(
            &SecureKeyStoreType::Encrypt,
            &KeyPair {
                public_key: self.get_encrypt_key_pair().get_public_key(),
                secret_key: self.get_encrypt_key_pair().get_secret_key(),
            },
        ) {
            Ok(_) => (),
            _ => panic!(),
        };

        let mut config = self.config.lock();
        config.save_did(did);
        config.save_is_initialized(true);
    }

    pub fn get_identifier(&self) -> Result<String, KeyPairingError> {
        self.config
            .lock()
            .get_did()
            .ok_or(KeyPairingError::DIDNotFound)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_create_keyring() {
        let keyring = match KeyPairing::create_keyring() {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(keyring.get_sign_key_pair().get_secret_key().len(), 32);
        assert_eq!(keyring.get_update_key_pair().get_secret_key().len(), 32);
        assert_eq!(keyring.get_recovery_key_pair().get_secret_key().len(), 32);
        assert_eq!(keyring.get_encrypt_key_pair().get_secret_key().len(), 32);
    }
}
