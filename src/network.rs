use home_config::HomeConfig;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;

use crate::nodex::errors::NodeXError;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct ConfigNetwork {
    secret_key: Option<String>,
    project_did: Option<String>,
}

#[derive(Debug)]
pub struct Network {
    config: HomeConfig,
    root: ConfigNetwork,
}

impl Network {
    fn touch(path: &Path) -> io::Result<()> {
        match OpenOptions::new().create(true).write(true).open(path) {
            Ok(mut file) => match file.write_all(b"{}") {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    pub fn new() -> Self {
        let config = HomeConfig::with_config_dir("nodex", "network.json");
        let config_dir = config.path().parent();

        if !Path::exists(config.path()) {
            match config_dir {
                Some(v) => {
                    match fs::create_dir_all(v) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{:?}", e);
                            panic!()
                        }
                    };
                }
                None => panic!(),
            };

            match Self::touch(config.path()) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{:?}", e);
                    panic!()
                }
            };
        }

        let root = match config.json::<ConfigNetwork>() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                panic!()
            }
        };

        Network { config, root }
    }

    pub fn write(&self) -> Result<(), NodeXError> {
        match self.config.save_json(&self.root) {
            Ok(v) => Ok(v),
            Err(e) => {
                log::error!("{:?}", e);
                panic!()
            }
        }
    }

    // NOTE: secret key
    pub fn get_secretk_key(&self) -> Option<String> {
        self.root.secret_key.clone()
    }

    pub fn save_secretk_key(&mut self, value: &str) {
        self.root.secret_key = Some(value.to_string());

        match self.write() {
            Ok(_) => {}
            Err(e) => {
                log::error!("{:?}", e);
                panic!()
            }
        }
    }

    // NOTE: project_did
    pub fn get_project_did(&self) -> Option<String> {
        self.root.project_did.clone()
    }

    pub fn save_project_did(&mut self, value: &str) {
        self.root.project_did = Some(value.to_string());

        match self.write() {
            Ok(_) => {}
            Err(e) => {
                log::error!("{:?}", e);
                panic!()
            }
        }
    }
}
