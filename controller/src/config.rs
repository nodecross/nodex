use lazy_static::lazy_static;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Config {
    pub config_dir: PathBuf,
    #[allow(dead_code)]
    pub nodex_dir: PathBuf,
    #[allow(dead_code)]
    pub runtime_dir: PathBuf,
    pub uds_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let config_dir = home_dir.join(".config").join("nodex");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        let nodex_dir = home_dir.join(".nodex");
        let runtime_dir = nodex_dir.join("run");

        fs::create_dir_all(&runtime_dir).expect("Failed to create runtime directory");

        let sock_path = runtime_dir.join("nodex.sock");

        Config {
            config_dir,
            nodex_dir,
            runtime_dir,
            uds_path: sock_path,
        }
    }
}

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::new());
}

pub fn get_config() -> &'static Mutex<Config> {
    &CONFIG
}
