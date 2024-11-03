use serde_json::Value;
use std::fs;
use std::error::Error;

pub fn execute(file: &str, field: &str, value: &str) -> Result<(), Box<dyn Error>> {
    println!("Updating JSON file '{}' field '{}' with value '{}'", file, field, value);

    let file_content = fs::read_to_string(file)?;
    let mut json_data: Value = serde_json::from_str(&file_content)?;

    let parts: Vec<&str> = field.split('.').collect();
    let mut current = &mut json_data;
    for part in &parts[..parts.len() - 1] {
        current = current.get_mut(part).ok_or("Invalid field path")?;
    }
    current[parts.last().unwrap()] = Value::String(value.to_string());

    fs::write(file, serde_json::to_string_pretty(&json_data)?)?;
    
    Ok(())
}
