mod move_action;
mod update_json;
mod command;

use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateYaml {
    pub version: String,
    pub description: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "action")]
pub enum Step {
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
    Command {
        description: String,
        command: String,
    },
}

pub fn run_actions(steps: Vec<Step>) -> Result<(), Box<dyn Error>> {
    for step in steps {
        match step {
            Step::Move {src, dest, .. } => {
                move_action::execute(&src, &dest)?;
            }
            Step::UpdateJson { file, field, value, .. } => {
                update_json::execute(&file, &field, &value)?;
            }
            Step::Command { command, .. } => {
                command::execute(&command)?;
            }
        }
    }
    Ok(())
}
