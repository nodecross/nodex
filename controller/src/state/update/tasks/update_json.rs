use serde_json::{error::Error as SerdeError, Value};
use std::fs;

#[derive(Debug, thiserror::Error)]
pub enum UpdateJsonError {
    #[error("Failed to read JSON file '{0}': {1}")]
    FileReadError(String, #[source] std::io::Error),
    #[error("Failed to parse JSON in file '{0}': {1}")]
    JsonParseError(String, #[source] SerdeError),
    #[error("Invalid field path '{0}'")]
    InvalidFieldPath(String),
    #[error("Failed to write JSON file '{0}': {1}")]
    FileWriteError(String, #[source] std::io::Error),
}

pub fn run(file: &String, field: &String, value: &String) -> Result<(), UpdateJsonError> {
    // Array updates are not supported.
    // It's unclear whether the operation is an addition or a completely new write.

    log::info!(
        "Updating JSON file '{}' field '{}' with value '{}'",
        file,
        field,
        value
    );

    let file_content = fs::read_to_string(file)
        .map_err(|e| UpdateJsonError::FileReadError(file.to_string(), e))?;

    let mut json_data: Value = serde_json::from_str(&file_content)
        .map_err(|e| UpdateJsonError::JsonParseError(file.to_string(), e))?;

    let parts: Vec<&str> = field.split('.').collect();
    let mut current = &mut json_data;
    for part in &parts[..parts.len() - 1] {
        current = current
            .get_mut(part)
            .ok_or_else(|| UpdateJsonError::InvalidFieldPath(field.to_string()))?;
    }

    current[parts.last().unwrap()] = Value::String(value.to_string());

    fs::write(
        file,
        serde_json::to_string_pretty(&json_data)
            .map_err(|e| UpdateJsonError::JsonParseError(file.to_string(), e))?,
    )
    .map_err(|e| UpdateJsonError::FileWriteError(file.to_string(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_run_handles_write_error() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, r#"{"key1": "value1"}"#).unwrap();

        let permissions = fs::Permissions::from_mode(0o444);
        fs::set_permissions(&file_path, permissions).unwrap();

        let field = "key1".to_string();
        let value = "new_value".to_string();
        let file_path_str = file_path.to_str().unwrap().to_string();

        let result = run(&file_path_str, &field, &value);
        assert!(
            matches!(result, Err(UpdateJsonError::FileWriteError(_, _))),
            "Expected FileWriteError, but got: {:?}",
            result
        );

        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o644)).unwrap();
    }

    #[test]
    fn test_run_creates_nested_structure() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, "{}").unwrap();

        let field = "key1.key2".to_string();
        let value = "new_value".to_string();
        let file_path_str = file_path.to_str().unwrap().to_string();

        let result = run(&file_path_str, &field, &value);
        assert!(
            result.is_ok(),
            "Expected run to succeed, but got an error: {:?}",
            result
        );

        let updated_content = fs::read_to_string(&file_path).unwrap();
        let expected_content = r#"{
  "key1": {
    "key2": "new_value"
  }
}"#;
        assert_eq!(
            updated_content.trim(),
            expected_content,
            "File content mismatch"
        );
    }

    #[test]
    fn test_run_invalid_field_path() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, r#"{"key1": {"other_key": "value1"}}"#).unwrap();

        let field = "key1.invalid_key".to_string();
        let value = "new_value".to_string();
        let file_path_str = file_path.to_str().unwrap().to_string();

        let result = run(&file_path_str, &field, &value);
        assert!(
            matches!(result, Err(UpdateJsonError::InvalidFieldPath(_))),
            "Expected InvalidFieldPath error, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_run_updates_existing_value() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, r#"{"key1": {"key2": "old_value"}}"#).unwrap();

        let field = "key1.key2".to_string();
        let value = "new_value".to_string();
        let file_path_str = file_path.to_str().unwrap().to_string();

        let result = run(&file_path_str, &field, &value);
        assert!(
            result.is_ok(),
            "Expected run to succeed, but got an error: {:?}",
            result
        );

        let updated_content = fs::read_to_string(&file_path).unwrap();
        let expected_content = r#"{
  "key1": {
    "key2": "new_value"
  }
}"#;
        assert_eq!(
            updated_content.trim(),
            expected_content,
            "File content mismatch"
        );
    }

    #[test]
    fn test_run_appends_to_array() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, r#"{"key1": {"key2": ["item1", "item2"]}}"#).unwrap();

        let field = "key1.key2".to_string();
        let value = "new_item".to_string();
        let file_path_str = file_path.to_str().unwrap().to_string();

        let result = run(&file_path_str, &field, &value);
        assert!(
            result.is_ok(),
            "Expected run to succeed, but got an error: {:?}",
            result
        );

        let updated_content = fs::read_to_string(&file_path).unwrap();
        let expected_content = r#"{
  "key1": {
    "key2": [
      "item1",
      "item2",
      "new_item"
    ]
  }
}"#;
        assert_eq!(
            updated_content.trim(),
            expected_content,
            "File content mismatch"
        );
    }
}
