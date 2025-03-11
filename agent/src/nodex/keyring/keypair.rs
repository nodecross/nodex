use crate::{
    config::SingletonAppConfig,
    nodex::extension::secure_keystore::{SecureKeyStore, SecureKeyStoreKey},
};
use protocol::keyring::keypair::{Ed25519KeyPair, K256KeyPair, X25519KeyPair};
use protocol::rand_core::OsRng;

use thiserror::Error;

pub struct KeyPairingWithConfig<S: SecureKeyStore> {
    sign: K256KeyPair,
    encrypt: X25519KeyPair,
    sign_time_series: Ed25519KeyPair,
    update: K256KeyPair,
    recovery: K256KeyPair,
    didwebvh_update: Ed25519KeyPair,
    didwebvh_recovery: Ed25519KeyPair,
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
        let sign_time_series = secure_keystore
            .read_sign_time_series()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let update = secure_keystore
            .read_update()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let recovery = secure_keystore
            .read_recovery()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let encrypt = secure_keystore
            .read_encrypt()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let didwebvh_update = secure_keystore
            .read_didwebvh_update()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let didwebvh_recovery = secure_keystore
            .read_didwebvh_recovery()
            .ok_or(KeyPairingError::KeyNotFound)?;

        Ok(KeyPairingWithConfig {
            sign,
            sign_time_series,
            update,
            recovery,
            encrypt,
            didwebvh_update,
            didwebvh_recovery,
            config,
            secure_keystore,
        })
    }

    pub fn create_keyring(config: Box<SingletonAppConfig>, secure_keystore: S) -> Self {
        // TODO: extension trng support
        let keyring = protocol::keyring::keypair::KeyPairing::create_keyring(OsRng);

        KeyPairingWithConfig {
            sign: keyring.sign,
            sign_time_series: keyring.sign_time_series,
            update: keyring.update,
            recovery: keyring.recovery,
            encrypt: keyring.encrypt,
            didwebvh_update: keyring.didwebvh_update,
            didwebvh_recovery: keyring.didwebvh_recovery,
            config,
            secure_keystore,
        }
    }

    pub fn get_keyring(&self) -> protocol::keyring::keypair::KeyPairing {
        protocol::keyring::keypair::KeyPairing {
            sign: self.sign.clone(),
            sign_time_series: self.sign_time_series.clone(),
            update: self.update.clone(),
            recovery: self.recovery.clone(),
            encrypt: self.encrypt.clone(),
            didwebvh_update: self.didwebvh_update.clone(),
            didwebvh_recovery: self.didwebvh_recovery.clone(),
        }
    }

    pub fn save(&mut self, did: &str) {
        self.secure_keystore
            .write(&SecureKeyStoreKey::Sign(&self.sign));
        self.secure_keystore
            .write(&SecureKeyStoreKey::SignTimeSeries(&self.sign_time_series));
        self.secure_keystore
            .write(&SecureKeyStoreKey::Update(&self.update));
        self.secure_keystore
            .write(&SecureKeyStoreKey::Recovery(&self.recovery));
        self.secure_keystore
            .write(&SecureKeyStoreKey::Encrypt(&self.encrypt));
        self.secure_keystore
            .write(&SecureKeyStoreKey::DidWebvhUpdate(&self.didwebvh_update));
        self.secure_keystore
            .write(&SecureKeyStoreKey::DidWebvhRecovery(
                &self.didwebvh_recovery,
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
