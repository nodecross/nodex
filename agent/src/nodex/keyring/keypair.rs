use crate::{
    config::SingletonAppConfig,
    nodex::extension::secure_keystore::{SecureKeyStore, SecureKeyStoreKey},
};
use protocol::keyring::keypair::{Ed25519KeyPair, K256KeyPair, X25519KeyPair};
use protocol::rand_core::OsRng;

use thiserror::Error;

pub struct KeyPairingWithConfig<S: SecureKeyStore> {
    sign: K256KeyPair,
    update: Ed25519KeyPair,
    next_key: Ed25519KeyPair,
    encrypt: X25519KeyPair,
    sidetree_update: K256KeyPair,
    sidetree_recovery: K256KeyPair,
    config: Box<SingletonAppConfig>,
    secure_keystore: S,
}

#[derive(Error, Debug)]
pub enum KeyPairingError {
    #[error("create keyring failed: {0}")]
    CreateKeyringFailed(#[from] protocol::keyring::keypair::KeyPairingError),
    #[error("key not found")]
    KeyNotFound,
    #[error("DID not found")]
    DIDNotFound,
}

impl<S: SecureKeyStore> KeyPairingWithConfig<S> {
    pub fn load_keyring(
        config: Box<SingletonAppConfig>,
        secure_keystore: S,
    ) -> Result<Self, KeyPairingError> {
        let sign = secure_keystore
            .read_sign()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let update = secure_keystore
            .read_update()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let next_key = secure_keystore
            .read_next_key()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let encrypt = secure_keystore
            .read_encrypt()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let sidetree_update = secure_keystore
            .read_sidetree_update()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let sidetree_recovery = secure_keystore
            .read_sidetree_recovery()
            .ok_or(KeyPairingError::KeyNotFound)?;

        Ok(KeyPairingWithConfig {
            sign,
            update,
            next_key,
            encrypt,
            sidetree_update,
            sidetree_recovery,
            config,
            secure_keystore,
        })
    }

    pub fn create_keyring(config: Box<SingletonAppConfig>, secure_keystore: S) -> Self {
        // TODO: extension trng support
        let keyring = protocol::keyring::keypair::KeyPairing::create_keyring(OsRng);

        KeyPairingWithConfig {
            sign: keyring.sign,
            update: keyring.update,
            next_key: keyring.next_key,
            encrypt: keyring.encrypt,
            sidetree_update: keyring.sidetree_update,
            sidetree_recovery: keyring.sidetree_recovery,
            config,
            secure_keystore,
        }
    }

    pub fn get_keyring(&self) -> protocol::keyring::keypair::KeyPairing {
        protocol::keyring::keypair::KeyPairing {
            sign: self.sign.clone(),
            update: self.update.clone(),
            next_key: self.next_key.clone(),
            encrypt: self.encrypt.clone(),
            sidetree_update: self.sidetree_update.clone(),
            sidetree_recovery: self.sidetree_recovery.clone(),
        }
    }

    pub fn save(&mut self, did: &str) {
        self.secure_keystore
            .write(&SecureKeyStoreKey::Sign(&self.sign));
        self.secure_keystore
            .write(&SecureKeyStoreKey::Update(&self.update));
        self.secure_keystore
            .write(&SecureKeyStoreKey::NextKey(&self.next_key));
        self.secure_keystore
            .write(&SecureKeyStoreKey::Encrypt(&self.encrypt));
        self.secure_keystore
            .write(&SecureKeyStoreKey::SidetreeUpdate(&self.sidetree_update));
        self.secure_keystore
            .write(&SecureKeyStoreKey::SidetreeRecovery(
                &self.sidetree_recovery,
            ));
        {
            let mut config = self.config.lock();
            config.save_did(did);
            config.save_is_initialized(true);
        }
    }

    pub fn get_identifier(&self) -> Result<String, KeyPairingError> {
        self.config
            .lock()
            .get_did()
            .ok_or(KeyPairingError::DIDNotFound)
    }
}
