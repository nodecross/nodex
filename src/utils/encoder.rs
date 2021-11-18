extern crate alloc;

use alloc::{string::String, vec::Vec};
use data_encoding::BASE64URL;

pub struct Encoder {}

impl Encoder {
    #[no_mangle]
    pub fn encode(content_vec: Vec<u8>) -> String {
        let content_u8: &[u8] = &content_vec[..];
        let content_base64url: String = BASE64URL.encode(content_u8);

        content_base64url
    }
}