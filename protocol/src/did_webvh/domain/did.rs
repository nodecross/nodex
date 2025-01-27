use super::crypto::hash::validate_hash;
use const_format::formatcp;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::str::FromStr;

static PAC_ENCODED: &str = r"%[0-9A-Fa-f]{2}";
static IDCHAR: &str = formatcp!(r"[a-zA-Z0-9.\-_{PAC_ENCODED}]");
static METHOD_CHAR: &str = r"[a-zA-Z0-9]";
static DID_PAT: &str = formatcp!(
    r"^did:(?<method_name>{METHOD_CHAR}+):(?<method_specific_id>({IDCHAR}*:)*{IDCHAR}+)$"
);

#[derive(Debug, Clone, PartialEq)]
pub enum DidError {
    InvalidMethod,
    InvaliDidentifier,
    InvalidFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Did {
    method: String,
    identifier: String,
}

impl Did {
    pub fn new(method: &str, identifier: &str) -> Result<Self, DidError> {
        if method.is_empty() {
            return Err(DidError::InvalidMethod);
        }
        if identifier.is_empty() {
            return Err(DidError::InvaliDidentifier);
        }
        if !method.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(DidError::InvalidMethod);
        }
        Ok(Self {
            method: method.to_string(),
            identifier: identifier.to_string(),
        })
    }

    pub fn get_method(&self) -> &str {
        &self.method
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    pub fn to_string(&self) -> String {
        format!("did:{}:{}", self.method, self.identifier)
    }
}

impl From<DidWebvh> for Did {
    fn from(did: DidWebvh) -> Self {
        Self {
            method: did.get_did().get_method().to_string(),
            identifier: did.get_did().get_identifier().to_string(),
        }
    }
}

impl FromStr for Did {
    type Err = DidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(DID_PAT).map_err(|_| DidError::InvalidFormat)?;
        let caps = re.captures(s).ok_or(DidError::InvalidFormat)?;
        let method = caps.name("method_name").unwrap().as_str();
        let identifier = caps.name("method_specific_id").unwrap().as_str();
        Self::new(method, identifier)
    }
}
impl Serialize for Did {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse()
            .map_err(|_| serde::de::Error::custom("Invalid Did Format"))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DidWebvhError {
    DidError(DidError),
    InvalidSCID,
    InvalidDomain,
    InvalidPath,
}

impl From<DidError> for DidWebvhError {
    fn from(err: DidError) -> Self {
        Self::DidError(err)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DidWebvh {
    did: Did,
    scid: String,
    uri: String,
}

impl DidWebvh {
    pub fn new(scid: &str, uri: &str) -> Result<Self, DidWebvhError> {
        if scid.is_empty() {
            return Err(DidWebvhError::InvalidSCID);
        }

        if !validate_hash(scid) {
            return Err(DidWebvhError::InvalidSCID);
        }

        if uri.is_empty() {
            return Err(DidWebvhError::InvalidDomain);
        }
        // uri is not contain '/', need to use ":" as path separator
        if uri.contains('/') {
            return Err(DidWebvhError::InvalidPath);
        }
        let did = Did::new("webvh", &format!("{}:{}", scid, uri))?;
        Ok(Self {
            did,
            scid: scid.to_string(),
            uri: uri.to_string(),
        })
    }

    pub fn get_scid(&self) -> &str {
        &self.scid
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn get_did(&self) -> &Did {
        &self.did
    }

    pub fn replace_scid(&self, scid: &str) -> Self {
        Self {
            did: Did::new("webvh", &format!("{}:{}", scid, self.uri)).unwrap(),
            scid: scid.to_string(),
            uri: self.uri.to_string(),
        }
    }

    pub fn did_to_https(&self) -> String {
        let mut parts: Vec<String> = self.uri.split(':').map(|s| s.to_string()).collect();
        if parts[0].contains("%3A") {
            parts[0] = parts[0].replace("%3A", ":");
        }
        format!("https://{}", parts.join("/"))
    }
}

impl TryFrom<Did> for DidWebvh {
    type Error = DidWebvhError;

    fn try_from(did: Did) -> Result<Self, Self::Error> {
        if did.get_method() != "webvh" {
            return Err(DidWebvhError::DidError(DidError::InvalidMethod));
        }
        let parts: Vec<&str> = did.get_identifier().split(':').collect();
        if parts.len() < 2 {
            return Err(DidWebvhError::InvalidPath);
        }
        Self::new(parts[0], parts[1..].join(":").as_str())
    }
}

impl FromStr for DidWebvh {
    type Err = DidWebvhError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let did: Did = s.parse()?;
        if did.get_method() != "webvh" {
            return Err(DidWebvhError::DidError(DidError::InvalidMethod));
        }
        let parts: Vec<&str> = did.get_identifier().split(':').collect();
        // parsed scid and domain
        // eg. ["scid", "example.com%3A8000", "path", "to", "resource"]
        if parts.len() < 2 {
            return Err(DidWebvhError::DidError(DidError::InvaliDidentifier));
        }
        // domain may be have path
        Self::new(parts[0], parts[1..].join(":").as_str())
    }
}

impl<'de> Deserialize<'de> for DidWebvh {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse()
            .map_err(|_| serde::de::Error::custom("Invalid Did Format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_example() -> Result<(), DidError> {
        let did = Did::new("example", "hogehoge")?;
        assert_eq!(did.get_method(), "example");
        assert_eq!(did.get_identifier(), "hogehoge");
        assert_eq!(did.to_string(), "did:example:hogehoge");

        let did = "did:example:hogehoge".parse::<Did>().unwrap();
        assert_eq!(did.get_method(), "example");
        assert_eq!(did.get_identifier(), "hogehoge");
        Ok(())
    }

    #[test]
    fn test_did_webvh_in_did() -> Result<(), DidError> {
        let did = Did::new(
            "webvh",
            "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com:path:to:resource",
        )?;
        assert_eq!(did.get_method(), "webvh");
        assert_eq!(
            did.get_identifier(),
            "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com:path:to:resource"
        );
        assert_eq!(
            did.to_string(),
            "did:webvh:QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com:path:to:resource"
        );

        let did = "did:webvh:QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com%3A8000:path:to:resource"
            .parse::<Did>()
            .unwrap();
        assert_eq!(did.get_method(), "webvh");
        assert_eq!(
            did.get_identifier(),
            "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com%3A8000:path:to:resource"
        );

        Ok(())
    }

    #[test]
    fn test_did_error() {
        assert_eq!(
            Did::new("", "example").unwrap_err(),
            DidError::InvalidMethod
        );
        assert_eq!(
            Did::new("web", "").unwrap_err(),
            DidError::InvaliDidentifier
        );
    }

    #[test]
    fn test_did_webvh() -> Result<(), DidWebvhError> {
        let did = DidWebvh::new(
            "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr",
            "example.com:path:to:resource",
        )?;
        assert_eq!(
            did.get_scid(),
            "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr"
        );
        assert_eq!(did.get_uri(), "example.com:path:to:resource");
        assert_eq!(did.did_to_https(), "https://example.com/path/to/resource");

        let did = "did:webvh:QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com%3A8000:path:to:resource"
            .parse::<DidWebvh>()
            .unwrap();
        assert_eq!(
            did.get_scid(),
            "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr"
        );
        assert_eq!(did.get_uri(), "example.com%3A8000:path:to:resource");
        assert_eq!(
            did.did_to_https(),
            "https://example.com:8000/path/to/resource"
        );

        let did = Did::new(
            "webvh",
            "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com:path:to:resource",
        )?;
        let did_webvh = DidWebvh::try_from(did)?;
        assert_eq!(
            did_webvh.get_scid(),
            "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr"
        );
        assert_eq!(did_webvh.get_uri(), "example.com:path:to:resource");
        assert_eq!(
            did_webvh.did_to_https(),
            "https://example.com/path/to/resource"
        );

        Ok(())
    }

    #[test]
    fn test_did_webvh_error() {
        assert_eq!(
            DidWebvh::new("", "path/to/resource").unwrap_err(),
            DidWebvhError::InvalidSCID
        );
        assert_eq!(
            DidWebvh::new("example.com%3A8000", "").unwrap_err(),
            DidWebvhError::InvalidSCID
        );
        assert_eq!(
            "did:QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:webvh:example.com%3A8000:path/to/resource"
                .parse::<DidWebvh>()
                .unwrap_err(),
            DidWebvhError::DidError(DidError::InvalidFormat)
        );
        assert_eq!(
            "did:webvh:example.com%3A8000"
                .parse::<DidWebvh>()
                .unwrap_err(),
            DidWebvhError::DidError(DidError::InvaliDidentifier)
        );
        assert_eq!(
            "did:webvh:QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com%3A8000:path/to/resource"
                .parse::<DidWebvh>()
                .unwrap_err(),
            DidWebvhError::DidError(DidError::InvalidFormat)
        );
    }
}
