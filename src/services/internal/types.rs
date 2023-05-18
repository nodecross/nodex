use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct VerifiedContainer {
    pub message: Value,
    pub metadata: Option<Value>,
}
