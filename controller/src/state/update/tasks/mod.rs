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
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    mod mock_move_resource {
        use super::*;
        #[allow(dead_code)]
        pub fn run(src: &str, _dest: &str) -> Result<(), MoveResourceError> {
            if src == "error" {
                Err(MoveResourceError::SourceNotFoundError(PathBuf::from(src)))
            } else {
                Ok(())
            }
        }
    }

    mod mock_update_json {
        use super::*;
        #[allow(dead_code)]
        pub fn run(file: &str, _field: &str, _value: &str) -> Result<(), UpdateJsonError> {
            if file == "error.json" {
                Err(UpdateJsonError::InvalidFieldPath(
                    "invalid_field".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }

    fn create_test_file(path: &str, content: &str) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn cleanup_test_file(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_handle_successful_move_tasks() {
        let source1_path = "/tmp/source1.txt";
        let source2_path = "/tmp/source2.txt";

        let dest1_path = "/tmp/dest1";
        let dest2_path = "/tmp/dest2";

        create_test_file(source1_path, "This is source1").expect("Failed to create source1.txt");
        create_test_file(source2_path, "This is source2").expect("Failed to create source2.txt");

        let tasks = vec![
            Task::Move {
                description: "Move file 1".to_string(),
                src: source1_path.to_string(),
                dest: dest1_path.to_string(),
            },
            Task::Move {
                description: "Move file 2".to_string(),
                src: source2_path.to_string(),
                dest: dest2_path.to_string(),
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

        assert!(
            fs::metadata(source1_path).is_err(),
            "Source1 file should have been moved"
        );
        assert!(
            fs::metadata(source2_path).is_err(),
            "Source2 file should have been moved"
        );
        assert!(
            fs::metadata(dest1_path).is_ok(),
            "Destination1 file should exist"
        );
        assert!(
            fs::metadata(dest2_path).is_ok(),
            "Destination2 file should exist"
        );

        cleanup_test_file(dest1_path);
        cleanup_test_file(dest2_path);
    }

    #[test]
    fn test_handle_successful_update_json_tasks() {
        let source1_path = "/tmp/test1.json";
        let source2_path = "/tmp/test2.json";

        create_test_file(
            source1_path,
            r#"{"key1": "old_value1", "key3": "unchanged"}"#,
        )
        .expect("Failed to create test1.json");
        create_test_file(
            source2_path,
            r#"{"key2": "old_value2", "key4": "unchanged"}"#,
        )
        .expect("Failed to create test2.json");

        let tasks = vec![
            Task::UpdateJson {
                description: "Update field 1".to_string(),
                file: source1_path.to_string(),
                field: "key1".to_string(),
                value: "value1".to_string(),
            },
            Task::UpdateJson {
                description: "Update field 2".to_string(),
                file: source2_path.to_string(),
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

        let updated_content1 =
            std::fs::read_to_string(source1_path).expect("Failed to read test1.json");
        let updated_content2 =
            std::fs::read_to_string(source2_path).expect("Failed to read test2.json");

        let json1: serde_json::Value =
            serde_json::from_str(&updated_content1).expect("Failed to parse test1.json");
        let json2: serde_json::Value =
            serde_json::from_str(&updated_content2).expect("Failed to parse test2.json");

        assert_eq!(json1["key1"], "value1", "key1 should be updated to value1");
        assert_eq!(json1["key3"], "unchanged", "key3 should remain unchanged");

        assert_eq!(json2["key2"], "value2", "key2 should be updated to value2");
        assert_eq!(json2["key4"], "unchanged", "key4 should remain unchanged");

        cleanup_test_file(source1_path);
        cleanup_test_file(source2_path);
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
