use serde_jcs;
use serde_json::{self, Value};

use crate::nodex::errors::NodeXError;

pub struct Jcs {}

impl Jcs {
    pub fn canonicalize(input: &str) -> Result<String, NodeXError> {
        let json = match serde_json::from_str::<Value>(input) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{})
        };

        match serde_jcs::to_string(&json) {
            Ok(v) => Ok(v),
            Err(_) => Err(NodeXError{})
        }
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
            Err(_) => panic!()
        };

        assert_eq!(result, r#"{"a":1,"b":[],"c":2}"#);
    }
}