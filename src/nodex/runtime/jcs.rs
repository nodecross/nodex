use serde_jcs;
use serde_json::{self, Value};

use thiserror::Error;

pub struct Jcs {}

#[derive(Debug, Error)]
pub enum JcsError {
    #[error("Decode failed")]
    DecodeFailed(serde_json::Error),
    #[error("Serialize failed")]
    SerializeFailed(serde_json::Error),
}

impl Jcs {
    pub fn canonicalize(input: &str) -> Result<String, JcsError> {
        let json = serde_json::from_str::<Value>(input).map_err(JcsError::DecodeFailed)?;

        serde_jcs::to_string(&json).map_err(JcsError::SerializeFailed)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn json() -> String {
        String::from(r#"{"c":2,"a":1,"b":[]}"#)
    }

    #[test]
    fn test_canonicalize() {
        let result = match Jcs::canonicalize(&json()) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result, r#"{"a":1,"b":[],"c":2}"#);
    }
}
