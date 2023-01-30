use serde_json::{Value, json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VerifiedContainer {
    pub message: Value,
    pub metadata: Option<Value>,
}