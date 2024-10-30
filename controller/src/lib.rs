use crate::config::get_config;
use crate::runtime::RuntimeInfo;
use state::manager::StateManager;

mod config;
mod process;
mod runtime;
mod state;

pub fn run() {
    let runtime_info_path = {
        let config = get_config().lock().unwrap();
        config.config_dir.join("runtime_info.json")
    };
    let mut runtime_info = RuntimeInfo::load_or_default(&runtime_info_path);

    let state_manager = StateManager::new();
    state_manager.handle_state(&mut runtime_info);

    runtime_info
        .write(&runtime_info_path)
        .expect("Failed to write runtime info");
}
