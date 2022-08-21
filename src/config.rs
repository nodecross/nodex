use serde::Deserialize;
use serde::Serialize;
use home_config::HomeConfig;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::io;
use std::fs;

use crate::unid::errors::UNiDError;

pub struct SignKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Clone, Deserialize, Serialize)]
struct SignKeyPairConfig {
    public_key: String,
    secret_key: String,
}

pub struct UpdateKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Clone, Deserialize, Serialize)]
struct UpdateKeyPairConfig {
    public_key: String,
    secret_key: String,
}

pub struct RecoverKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Clone, Deserialize, Serialize)]
struct RecoverKeyPairConfig {
    public_key: String,
    secret_key: String,
}

pub struct EncryptKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Clone, Deserialize, Serialize)]
struct EncryptKeyPairConfig {
    public_key: String,
    secret_key: String,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ConfigRoot {
    did: Option<String>,
    mnemonic: Option<String>,
    sign: Option<SignKeyPairConfig>,
    update: Option<UpdateKeyPairConfig>,
    recover: Option<RecoverKeyPairConfig>,
    encrypt: Option<EncryptKeyPairConfig>,
    is_initialized: bool,
    schema_version: u8,
}

impl Default for ConfigRoot {
    fn default() -> Self {
        ConfigRoot {
            did: None,
            mnemonic: None,
            sign: None,
            update: None,
            recover: None,
            encrypt: None,
            is_initialized: false,
            schema_version: 1,
        }
    }
}

pub struct AppConfig {
    config: HomeConfig,
    root: ConfigRoot
}

impl AppConfig {
    fn touch(path: &Path) -> io::Result<()> {
        match OpenOptions::new().create(true).write(true).open(path) {
            Ok(mut file) => {
                match file.write_all(b"{}") {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err)
                }
            },
            Err(err) => Err(err),
        }
    }

    pub fn new() -> Self {
        let config = HomeConfig::new("unid", "config");
        let config_dir = config.path().parent();

        if ! Path::exists(config.path()) {
            match config_dir {
                Some(v) => {
                    match fs::create_dir(&v) {
                        Ok(_) => {},
                        Err(_) => panic!()
                    };
                },
                None => panic!()
            };

            match Self::touch(&config.path()) {
                Ok(_) => {},
                Err(_) => panic!()
            };
        }

        let root = match config.json::<ConfigRoot>() {
            Ok(v) => v,
            Err(err) => {
                println!("{:?}", err);
                panic!()
            }
        };

        AppConfig { root, config }
    }

    pub fn write(&self) -> Result<(), UNiDError> {
        match self.config.save_json(&self.root) {
            Ok(v) => Ok(v),
            Err(err) => {
                println!("{:?}", err);
                panic!()
            }
        }
    }

    pub fn encode(&self, value: &Option<Vec<u8>>) -> Option<String> {
        match value {
            Some(v) => {
                Some(hex::encode(&v))
            },
            None => None,
        }
    }

    pub fn decode(&self, value: &Option<String>) -> Option<Vec<u8>> {
        match value {
            Some(v) => {
                match hex::decode(&v) {
                    Ok(v) => Some(v),
                    Err(_) => None,
                }
            },
            None => None,
        }
    }

    // NOTE: SIGN
    pub fn load_sign_key_pair(&self) -> Option<SignKeyPair> {
        match self.root.sign.clone() {
            Some(v) => {
                let pk = match self.decode(&Some(v.public_key)) {
                    Some(v) => v,
                    None => return None,
                };
                let sk = match self.decode(&Some(v.secret_key)) {
                    Some(v) => v,
                    None => return None,
                };

                Some(SignKeyPair { public_key: pk, secret_key: sk })
            },
            None => None,
        }
    }

    pub fn save_sign_key_pair(&mut self, value: &SignKeyPair) {
        let pk = match self.encode(&Some(value.public_key.clone())) {
            Some(v) => v,
            None => return,
        };
        let sk = match self.encode(&Some(value.secret_key.clone())) {
            Some(v) => v,
            None => return,
        };

        self.root.sign = Some(SignKeyPairConfig {
            public_key: pk,
            secret_key: sk,
        });

        match self.write() {
            Ok(_) => {},
            Err(_) => panic!()
        }
    }

    // NOTE: UPDATE
    pub fn load_update_key_pair(&self) -> Option<UpdateKeyPair> {
        match self.root.update.clone() {
            Some(v) => {
                let pk = match self.decode(&Some(v.public_key)) {
                    Some(v) => v,
                    None => return None,
                };
                let sk = match self.decode(&Some(v.secret_key)) {
                    Some(v) => v,
                    None => return None,
                };

                Some(UpdateKeyPair { public_key: pk, secret_key: sk })
            },
            None => None,
        }
    }

    pub fn save_update_key_pair(&mut self, value: &UpdateKeyPair) {
        let pk = match self.encode(&Some(value.public_key.clone())) {
            Some(v) => v,
            None => return,
        };
        let sk = match self.encode(&Some(value.secret_key.clone())) {
            Some(v) => v,
            None => return,
        };

        self.root.update = Some(UpdateKeyPairConfig {
            public_key: pk,
            secret_key: sk,
        });

        match self.write() {
            Ok(_) => {},
            Err(_) => panic!()
        }
    }

    // NOTE: RECOVER
    pub fn load_recovery_key_pair(&self) -> Option<RecoverKeyPair> {
        match self.root.recover.clone() {
            Some(v) => {
                let pk = match self.decode(&Some(v.public_key)) {
                    Some(v) => v,
                    None => return None,
                };
                let sk = match self.decode(&Some(v.secret_key)) {
                    Some(v) => v,
                    None => return None,
                };

                Some(RecoverKeyPair { public_key: pk, secret_key: sk })
            },
            None => None,
        }
    }

    pub fn save_recover_key_pair(&mut self, value: &RecoverKeyPair) {
        let pk = match self.encode(&Some(value.public_key.clone())) {
            Some(v) => v,
            None => return,
        };
        let sk = match self.encode(&Some(value.secret_key.clone())) {
            Some(v) => v,
            None => return,
        };

        self.root.recover = Some(RecoverKeyPairConfig {
            public_key: pk,
            secret_key: sk,
        });

        match self.write() {
            Ok(_) => {},
            Err(_) => panic!()
        }
    }

    // NOTE: ENCRYPT
    pub fn load_encrypt_key_pair(&self) -> Option<EncryptKeyPair> {
        match self.root.encrypt.clone() {
            Some(v) => {
                let pk = match self.decode(&Some(v.public_key)) {
                    Some(v) => v,
                    None => return None,
                };
                let sk = match self.decode(&Some(v.secret_key)) {
                    Some(v) => v,
                    None => return None,
                };

                Some(EncryptKeyPair { public_key: pk, secret_key: sk })
            },
            None => None,
        }
    }

    pub fn save_encrypt_key_pair(&mut self, value: &EncryptKeyPair) {
        let pk = match self.encode(&Some(value.public_key.clone())) {
            Some(v) => v,
            None => return,
        };
        let sk = match self.encode(&Some(value.secret_key.clone())) {
            Some(v) => v,
            None => return,
        };

        self.root.encrypt = Some(EncryptKeyPairConfig {
            public_key: pk,
            secret_key: sk,
        });

        match self.write() {
            Ok(_) => {},
            Err(_) => panic!()
        }
    }

    // NOTE: DID
    pub fn get_did(&self) -> Option<String> {
        self.root.did.clone()
    }

    pub fn save_did(&mut self, value: &str) {
        self.root.did = Some(value.clone().to_string());

        match self.write() {
            Ok(_) => {},
            Err(_) => panic!()
        }
    }

    // NOTE: Mnemonic
    pub fn get_mnemonic(&self) -> Option<String> {
        self.root.mnemonic.clone()
    }

    pub fn save_mnemonic(&mut self, value: &str) {
        self.root.mnemonic = Some(value.clone().to_string());

        match self.write() {
            Ok(_) => {},
            Err(_) => panic!()
        }
    }

    // NOTE: Is Initialized
    pub fn get_is_initialized(&self) -> bool {
        self.root.is_initialized
    }

    pub fn save_is_initialized(&mut self, value: bool) {
        self.root.is_initialized = value.clone();
        match self.write() {
            Ok(_) => {},
            Err(_) => panic!()
        }
    }
}