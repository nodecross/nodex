use libsecp256k1::{PublicKey, PublicKeyFormat};

pub struct Secp256k1 {}

const COMPRESSED_PUBLIC_KEY_SIZE: usize = 33; // Buffer(0x04 + PublicKey (32 = 256 bit))
const UNCOMPRESSED_PUBLIC_KEY_SIZE: usize = 65; // Buffer(0x04 + PublicKey (64 = 512 bit))

impl Secp256k1 {
    pub fn public_key_convert(pub_key_vec: Vec<u8>, compressed: bool) -> Vec<u8> {
        let pub_key_u8: &[u8] = &pub_key_vec[..];
        let pub_key_pk: PublicKey;
        let serialized_pub_key_vec: Vec<u8>;

        match pub_key_vec.len() {
            COMPRESSED_PUBLIC_KEY_SIZE => {
                pub_key_pk = PublicKey::parse_slice(pub_key_u8, Some(PublicKeyFormat::Compressed)).unwrap();
            }

            UNCOMPRESSED_PUBLIC_KEY_SIZE => {
                pub_key_pk = PublicKey::parse_slice(pub_key_u8, Some(PublicKeyFormat::Compressed)).unwrap();
            }

            _ => panic!("public key length is invalid"),
        }

        if compressed {
            let serialized_key = pub_key_pk.serialize_compressed();
            let serialized_key_u8: &[u8] = &serialized_key;

            serialized_pub_key_vec = serialized_key_u8.to_vec();
        } else {
            let serialized_key = pub_key_pk.serialize();
            let serialized_key_u8: &[u8] = &serialized_key;

            serialized_pub_key_vec = serialized_key_u8.to_vec();
        }

        serialized_pub_key_vec
    }
}
