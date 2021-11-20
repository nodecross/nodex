extern crate alloc;

use alloc::{string::String, vec::Vec};
use data_encoding::BASE64URL;

pub struct Codec {}

impl Codec {
    pub fn base64_encode(content: String) -> String {
        BASE64URL.encode(content.as_bytes())
    }

    pub fn base64_decode(content: String) -> Vec<u8> {
        BASE64URL.decode(content.as_bytes()).unwrap()
    }
}