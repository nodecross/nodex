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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::update::tasks::{move_resource, update_json};
    use std::path::PathBuf;

    mod mock_move_resource {
        use super::*;
        pub fn run(src: &String, dest: &String) -> Result<(), MoveResourceError> {
            if src == "error" {
                Err(MoveResourceError::SourceNotFoundError(PathBuf::from(src)))
            } else {
                Ok(())
            }
        }
    }

    mod mock_update_json {
        use super::*;
        pub fn run(file: &String, field: &String, value: &String) -> Result<(), UpdateJsonError> {
            if file == "error.json" {
                Err(UpdateJsonError::InvalidFieldPath(
                    "invalid_field".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_handle_successful_move_tasks() {
        let tasks = vec![
            Task::Move {
                description: "Move file 1".to_string(),
                src: "/tmp/source1.txt".to_string(),
                dest: "/tmp/dest1".to_string(),
            },
            Task::Move {
                description: "Move file 2".to_string(),
                src: "/tmp/source2.txt".to_string(),
                dest: "/tmp/dest2".to_string(),
            },
        ];

        let action = UpdateAction {
            version: "1.0.0".to_string(),
            description: "Test move tasks".to_string(),
            tasks,
        };

        let result = action.handle();
        assert!(
            result.is_ok(),
            "Expected successful execution, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_handle_successful_update_json_tasks() {
        let tasks = vec![
            Task::UpdateJson {
                description: "Update field 1".to_string(),
                file: "/tmp/test1.json".to_string(),
                field: "key1".to_string(),
                value: "value1".to_string(),
            },
            Task::UpdateJson {
                description: "Update field 2".to_string(),
                file: "/tmp/test2.json".to_string(),
                field: "key2".to_string(),
                value: "value2".to_string(),
            },
        ];

        let action = UpdateAction {
            version: "1.0.0".to_string(),
            description: "Test update JSON tasks".to_string(),
            tasks,
        };

        let result = action.handle();
        assert!(
            result.is_ok(),
            "Expected successful execution, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_handle_move_task_error() {
        let tasks = vec![
            Task::Move {
                description: "Move valid file".to_string(),
                src: "/tmp/source1.txt".to_string(),
                dest: "/tmp/dest1".to_string(),
            },
            Task::Move {
                description: "Move invalid file".to_string(),
                src: "error".to_string(),
                dest: "/tmp/dest2".to_string(),
            },
        ];

        let action = UpdateAction {
            version: "1.0.0".to_string(),
            description: "Test move task error".to_string(),
            tasks,
        };

        let result = action.handle();
        assert!(
            matches!(result, Err(UpdateActionError::Move(_))),
            "Expected Move error, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_handle_update_json_task_error() {
        let tasks = vec![
            Task::UpdateJson {
                description: "Update valid JSON".to_string(),
                file: "/tmp/test1.json".to_string(),
                field: "key1".to_string(),
                value: "value1".to_string(),
            },
            Task::UpdateJson {
                description: "Update invalid JSON".to_string(),
                file: "error.json".to_string(),
                field: "key2".to_string(),
                value: "value2".to_string(),
            },
        ];

        let action = UpdateAction {
            version: "1.0.0".to_string(),
            description: "Test update JSON task error".to_string(),
            tasks,
        };

        let result = action.handle();
        assert!(
            matches!(result, Err(UpdateActionError::UpdateJson(_))),
            "Expected UpdateJson error, but got: {:?}",
            result
        );
    }
}
