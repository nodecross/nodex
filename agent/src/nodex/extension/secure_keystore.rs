use protocol::keyring::keypair::{K256KeyPair, X25519KeyPair};

use crate::config::SingletonAppConfig;

pub enum SecureKeyStoreKey<'a> {
    Sign(&'a K256KeyPair),
    Update(&'a K256KeyPair),
    Recovery(&'a K256KeyPair),
    Encrypt(&'a X25519KeyPair),
}

#[derive(Debug)]
pub enum SecureKeyStoreType {
    Sign,
    Update,
    Recovery,
    Encrypt,
}

pub trait SecureKeyStore {
    fn write(&self, key_pair: &SecureKeyStoreKey);
    fn read_sign(&self) -> Option<K256KeyPair>;
    fn read_update(&self) -> Option<K256KeyPair>;
    fn read_recovery(&self) -> Option<K256KeyPair>;
    fn read_encrypt(&self) -> Option<X25519KeyPair>;
}

#[derive(Clone)]
pub struct FileBaseKeyStore {
    config: Box<SingletonAppConfig>,
}

impl FileBaseKeyStore {
    pub fn new(config: Box<SingletonAppConfig>) -> Self {
        FileBaseKeyStore { config }
    }
}

fn k2t(k: &SecureKeyStoreKey) -> SecureKeyStoreType {
    match k {
        SecureKeyStoreKey::Sign(_) => SecureKeyStoreType::Sign,
        SecureKeyStoreKey::Update(_) => SecureKeyStoreType::Update,
        SecureKeyStoreKey::Recovery(_) => SecureKeyStoreType::Recovery,
        SecureKeyStoreKey::Encrypt(_) => SecureKeyStoreType::Encrypt,
    }
}

impl SecureKeyStore for FileBaseKeyStore {
    fn write(&self, key_pair: &SecureKeyStoreKey) {
        log::info!("Called: write_internal (type: {:?})", k2t(key_pair));

        let mut config = self.config.lock();

        match key_pair {
            SecureKeyStoreKey::Sign(k) => config.save_sign_key_pair(k),
            SecureKeyStoreKey::Update(k) => config.save_update_key_pair(k),
            SecureKeyStoreKey::Recovery(k) => config.save_recovery_key_pair(k),
            SecureKeyStoreKey::Encrypt(k) => config.save_encrypt_key_pair(k),
        };
    }

    fn read_sign(&self) -> Option<K256KeyPair> {
        log::debug!("Called: read_internal (type: sign)");
        let config = self.config.lock();
        config.load_sign_key_pair()
    }
    fn read_update(&self) -> Option<K256KeyPair> {
        log::debug!("Called: read_internal (type: update)");
        let config = self.config.lock();
        config.load_update_key_pair()
    }
    fn read_recovery(&self) -> Option<K256KeyPair> {
        log::debug!("Called: read_internal (type: recovery)");
        let config = self.config.lock();
        config.load_recovery_key_pair()
    }
    fn read_encrypt(&self) -> Option<X25519KeyPair> {
        log::debug!("Called: read_internal (type: encrypt)");
        let config = self.config.lock();
        config.load_encrypt_key_pair()
    }
}
