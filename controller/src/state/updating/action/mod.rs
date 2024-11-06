mod move_action;
mod update_json;

use crate::state::updating::action::{
    move_action::MoveOperationError, update_json::UpdateJsonOperationError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateAction {
    pub version: String,
    pub description: String,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "action")]
pub enum Operation {
    Move {
        description: String,
        src: String,
        dest: String,
    },
    UpdateJson {
        description: String,
        file: String,
        field: String,
        value: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateActionError {
    #[error("Move operation failed: {0}")]
    Move(#[from] MoveOperationError),
    #[error("Update JSON operation failed: {0}")]
    UpdateJson(#[from] UpdateJsonOperationError),
}

impl UpdateAction {
    pub fn run(&self) -> Result<(), UpdateActionError> {
        for operation in &self.operations {
            match operation {
                Operation::Move { src, dest, .. } => {
                    move_action::execute(src, dest)?;
                }
                Operation::UpdateJson {
                    file, field, value, ..
                } => {
                    update_json::execute(file, field, value)?;
                }
            };
        }
        Ok(())
    }
}
