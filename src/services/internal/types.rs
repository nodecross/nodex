use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VerifiedContainer {
    pub message: Value,
    pub metadata: Option<Value>,
}