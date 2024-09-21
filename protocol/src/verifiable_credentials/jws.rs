use std::convert::TryInto;

use data_encoding::BASE64URL_NOPAD;
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

// TODO: Design the interface to have an implementation with accelerators.

#[derive(Debug, Serialize, Deserialize)]
struct JwsHeader {
    alg: String,
    b64: bool,
    crit: Vec<String>,
}

#[derive(Debug, Error)]
pub enum JwsEncodeError {
    #[error("PublicKeyConvertError : {0:?}")]
    SignatureError(#[from] k256::ecdsa::Error),
    #[error("CanonicalizeError : {0:?}")]
    CanonicalizeError(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum JwsDecodeError {
    #[error("DecodeError: {0:?}")]
    DecodeError(#[from] data_encoding::DecodeError),
    #[error(transparent)]
    JsonParseError(#[from] serde_json::Error),
    #[error("invalid signature length: {0}")]
    InvalidSignatureLength(usize),
    #[error("InvalidAlgorithm: {0}")]
    InvalidAlgorithm(String),
    #[error("b64 option is not supported")]
    B64NotSupported,
    #[error("b64 option is not supported, but contained")]
    B64NotSupportedButContained,
    #[error("EmptyPayload")]
    EmptyPayload,
    #[error("InvalidJws : {0}")]
    InvalidJws(String),
    #[error("CryptError: {0:?}")]
    CryptError(#[from] k256::ecdsa::Error),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

pub fn sign(object: &Value, secret_key: &k256::SecretKey) -> Result<String, JwsEncodeError> {
    // NOTE: header
    let header = JwsHeader { alg: "ES256K".to_string(), b64: false, crit: vec!["b64".to_string()] };
    let header = serde_jcs::to_string(&header)?;
    let header = BASE64URL_NOPAD.encode(header.as_bytes());
    // NOTE: payload
    let payload = BASE64URL_NOPAD.encode(object.to_string().as_bytes());
    // NOTE: message
    let message = [header.clone(), payload].join(".");
    let message: &[u8] = message.as_bytes();

    // NOTE: signature
    let signing_key: SigningKey = secret_key.into();
    let signature: Signature = signing_key.try_sign(message)?;
    let signature = BASE64URL_NOPAD.encode(&signature.to_vec());

    Ok([header, "".to_string(), signature].join("."))
}

pub fn verify(
    object: &Value,
    jws: &str,
    public_key: &k256::PublicKey,
) -> Result<(), JwsDecodeError> {
    let split: Vec<String> = jws.split('.').map(|v| v.to_string()).collect();

    if split.len() != 3 {
        return Err(JwsDecodeError::InvalidJws(jws.to_string()));
    }

    let _header = split[0].clone();
    let __payload = split[1].clone();
    let _signature = split[2].clone();

    // NOTE: header
    let decoded = BASE64URL_NOPAD.decode(_header.as_bytes())?;
    let decoded = String::from_utf8(decoded)?;
    let header = serde_json::from_str::<JwsHeader>(&decoded)?;

    if header.alg != *"ES256K" {
        return Err(JwsDecodeError::InvalidAlgorithm(header.alg));
    }
    if header.b64 {
        return Err(JwsDecodeError::B64NotSupported);
    }
    if header.crit.iter().all(|v| v != "b64") {
        return Err(JwsDecodeError::B64NotSupportedButContained);
    };

    // NOTE: payload
    if __payload != *"".to_string() {
        return Err(JwsDecodeError::EmptyPayload);
    }
    let _payload = BASE64URL_NOPAD.encode(object.to_string().as_bytes());

    // NOTE: message
    let message = [_header, _payload].join(".");

    // NOTE: signature
    let signature = BASE64URL_NOPAD.decode(_signature.as_bytes())?;
    if signature.len() != 64 {
        return Err(JwsDecodeError::InvalidSignatureLength(signature.len()));
    }
    let r: &[u8; 32] = &signature[0..32].try_into().unwrap();
    let s: &[u8; 32] = &signature[32..].try_into().unwrap();
    let wrapped_signature = Signature::from_scalars(*r, *s)?;

    let verify_key = VerifyingKey::from(public_key);
    Ok(verify_key.verify(message.as_bytes(), &wrapped_signature)?)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    const SECRET_KEY: [u8; 32] = [
        0xc7, 0x39, 0x80, 0x5a, 0xb0, 0x3d, 0xa6, 0x2d, 0xdb, 0xe0, 0x33, 0x90, 0xac, 0xdf, 0x76,
        0x15, 0x64, 0x0a, 0xa6, 0xed, 0x31, 0xb8, 0xf1, 0x82, 0x43, 0xf0, 0x4a, 0x57, 0x2c, 0x52,
        0x8e, 0xdb,
    ];

    const PUBLIC_KEY: [u8; 33] = [
        0x02, 0x70, 0x96, 0x45, 0x32, 0xf0, 0x83, 0xf4, 0x5f, 0xe8, 0xe8, 0xcc, 0xea, 0x96, 0xa2,
        0x2f, 0x60, 0x18, 0xd4, 0x6a, 0x40, 0x6f, 0x58, 0x3a, 0xb2, 0x26, 0xb1, 0x92, 0x83, 0xaa,
        0x60, 0x5c, 0x44,
    ];

    fn message() -> String {
        String::from(r#"{"k":"0123456789abcdef"}"#)
    }

    fn signature() -> String {
        String::from(
            "eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..vuhCrs1zs9Mlhof0TXgN9XQEY9ZJ2g2kZsH4Ef99wn5MR0pQOhkAHvgYZfHBvXOR795WnWKF_rUiE85abp5CAA",
        )
    }

    #[test]
    pub fn test_encode() {
        let sk = k256::SecretKey::from_slice(&SECRET_KEY).unwrap();
        let json: Value = serde_json::from_str(&message()).unwrap();
        let result = sign(&json, &sk).unwrap();

        assert_eq!(result, signature())
    }

    #[test]
    pub fn test_verify() {
        let pk = k256::PublicKey::from_sec1_bytes(&PUBLIC_KEY).unwrap();
        let json: Value = serde_json::from_str(&message()).unwrap();
        verify(&json, &signature(), &pk).unwrap();
    }
}
