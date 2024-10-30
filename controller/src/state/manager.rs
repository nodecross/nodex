use crate::runtime::{RuntimeInfo, State};
use crate::state::{default::DefaultState, updating::UpdatingState};

pub struct StateManager;

impl StateManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_state(&self, runtime_info: &mut RuntimeInfo) {
        match runtime_info.state {
            State::Updating => {
                let updating_state = UpdatingState {};
                updating_state.handle();
            }
            State::Default => {
                let default_state = DefaultState {};
                default_state.handle(runtime_info);
            }
        }
    }
}
