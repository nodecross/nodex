use protocol::keyring::keypair::{Ed25519KeyPair, K256KeyPair, X25519KeyPair};

use crate::config::SingletonAppConfig;

pub enum SecureKeyStoreKey<'a> {
    Sign(&'a K256KeyPair),
    Update(&'a Ed25519KeyPair),
    NextKey(&'a Ed25519KeyPair),
    Encrypt(&'a X25519KeyPair),
    SidetreeUpdate(&'a K256KeyPair),
    SidetreeRecovery(&'a K256KeyPair),
}

#[derive(Debug)]
pub enum SecureKeyStoreType {
    Sign,
    Update,
    NextKey,
    Encrypt,
    SidetreeUpdate,
    SidetreeRecovery,
}

pub trait SecureKeyStore {
    fn write(&self, key_pair: &SecureKeyStoreKey);
    fn read_sign(&self) -> Option<K256KeyPair>;
    fn read_update(&self) -> Option<Ed25519KeyPair>;
    fn read_next_key(&self) -> Option<Ed25519KeyPair>;
    fn read_encrypt(&self) -> Option<X25519KeyPair>;
    fn read_sidetree_update(&self) -> Option<K256KeyPair>;
    fn read_sidetree_recovery(&self) -> Option<K256KeyPair>;
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
        SecureKeyStoreKey::NextKey(_) => SecureKeyStoreType::NextKey,
        SecureKeyStoreKey::Encrypt(_) => SecureKeyStoreType::Encrypt,
        SecureKeyStoreKey::SidetreeUpdate(_) => SecureKeyStoreType::SidetreeUpdate,
        SecureKeyStoreKey::SidetreeRecovery(_) => SecureKeyStoreType::SidetreeRecovery,
    }
}

impl SecureKeyStore for FileBaseKeyStore {
    fn write(&self, key_pair: &SecureKeyStoreKey) {
        log::info!("Called: write_internal (type: {:?})", k2t(key_pair));

        let mut config = self.config.lock();

        match key_pair {
            SecureKeyStoreKey::Sign(k) => config.save_sign_key_pair(k),
            SecureKeyStoreKey::Update(k) => config.save_update_key_pair(k),
            SecureKeyStoreKey::NextKey(k) => config.save_next_key_pair(k),
            SecureKeyStoreKey::Encrypt(k) => config.save_encrypt_key_pair(k),
            SecureKeyStoreKey::SidetreeUpdate(k) => config.save_sidetree_update_key_pair(k),
            SecureKeyStoreKey::SidetreeRecovery(k) => config.save_sidetree_recovery_key_pair(k),
        };
    }

    fn read_sign(&self) -> Option<K256KeyPair> {
        log::debug!("Called: read_internal (type: sign)");
        let config = self.config.lock();
        config.load_sign_key_pair()
    }
    fn read_update(&self) -> Option<Ed25519KeyPair> {
        log::debug!("Called: read_internal (type: update)");
        let config = self.config.lock();
        config.load_update_key_pair()
    }
    fn read_next_key(&self) -> Option<Ed25519KeyPair> {
        log::debug!("Called: read_internal (type: recovery)");
        let config = self.config.lock();
        config.load_next_key_pair()
    }
    fn read_encrypt(&self) -> Option<X25519KeyPair> {
        log::debug!("Called: read_internal (type: encrypt)");
        let config = self.config.lock();
        config.load_encrypt_key_pair()
    }
    fn read_sidetree_update(&self) -> Option<K256KeyPair> {
        log::debug!("Called: read_internal (type: sidetree_update)");
        let config = self.config.lock();
        config.load_sidetree_update_key_pair()
    }
    fn read_sidetree_recovery(&self) -> Option<K256KeyPair> {
        log::debug!("Called: read_internal (type: sidetree_recovery)");
        let config = self.config.lock();
        config.load_sidetree_recovery_key_pair()
    }
}
