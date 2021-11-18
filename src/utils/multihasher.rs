use super::encoder::*;
use multihash::{Code, Multihash, MultihashDigest};
use olpc_cjson::CanonicalFormatter;
use serde::Serialize;
use sha2::{Digest, Sha256};
pub struct Multihasher {}

impl Multihasher {
    pub fn hash_as_non_multihash_buffer(content_vec: Vec<u8>) -> Vec<u8> {
        let content_u8: &[u8] = &content_vec[..];
        let digested = Sha256::digest(content_u8);
        let digested_vec: Vec<u8> = digested.to_vec();

        digested_vec
    }

    pub fn hash(content_vec: Vec<u8>) -> Vec<u8> {
        let content_u8: &[u8] = &content_vec[..];
        let digested: Multihash = Code::Sha2_256.digest(content_u8);
        let digested_vec: Vec<u8> = digested.to_bytes();

        digested_vec
    }

    pub fn canonicalize_then_double_hash_then_encode(content_serde: serde_json::Value) -> String {
        let mut content_buf = Vec::new();
        let mut ser =
            serde_json::Serializer::with_formatter(&mut content_buf, CanonicalFormatter::new());
        content_serde.serialize(&mut ser).unwrap();
        let intermediate_hash_buffer: Vec<u8> = Multihasher::hash_as_non_multihash_buffer(content_buf);

        Multihasher::hash_then_encode(intermediate_hash_buffer)
    }

    pub fn hash_then_encode(content_vec: Vec<u8>) -> String {
        let multihash_buf: Vec<u8> = Multihasher::hash(content_vec);
        let encoded_base64_string: String = Encoder::encode(multihash_buf);

        encoded_base64_string
    }
}
