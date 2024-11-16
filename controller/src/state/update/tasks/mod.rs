mod move_resource;
mod update_json;

use crate::state::update::tasks::{move_resource::MoveResourceError, update_json::UpdateJsonError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateAction {
    pub version: String,
    pub description: String,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "action")]
pub enum Task {
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
    #[error("Move task failed: {0}")]
    Move(#[from] MoveResourceError),
    #[error("Update JSON operation failed: {0}")]
    UpdateJson(#[from] UpdateJsonError),
}

impl UpdateAction {
    pub fn handle(&self) -> Result<(), UpdateActionError> {
        for task in &self.tasks {
            match task {
                Task::Move { src, dest, .. } => {
                    move_resource::run(src, dest)?;
                }
                Task::UpdateJson {
                    file, field, value, ..
                } => {
                    update_json::run(file, field, value)?;
                }
            };
        }
        Ok(())
    }
}
