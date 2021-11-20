use super::interfaces::keyring_model::Secp256k1;
use crate::core::interfaces::did_document::*;
use crate::core::interfaces::did_operation::*;

pub struct Secp256k1Context {
    pub public: Vec<u8>,
    pub private: Vec<u8>,
}

pub struct Secp256k1HexKeyPair {
    pub public: String,
    pub private: String,
}

const PRIVATE_KEY_SIZE: usize = 32; // Buffer(PrivateKey (32 = 256 bit))
const COMPRESSED_PUBLIC_KEY_SIZE: usize = 33; // Buffer(0x04 + PublicKey (32 = 256 bit))
const UNCOMPRESSED_PUBLIC_KEY_SIZE: usize = 65; // Buffer(0x04 + PublicKey (64 = 512 bit))

impl Secp256k1 {
    pub fn new(context: Secp256k1Context) -> Self {
        let private: Vec<u8>;
        let public: Vec<u8>;
        assert_eq!(context.private.len(), PRIVATE_KEY_SIZE);
        private = context.private;
        match context.public.len() {
            COMPRESSED_PUBLIC_KEY_SIZE => {
                public = Secp256k1::transform_uncompressed_public_key(context.public);
            }
            UNCOMPRESSED_PUBLIC_KEY_SIZE => {
                public = context.public;
            }
            _ => panic!("public key length is invalid"),
        };
        Secp256k1 {
            _public: public,
            _private: private,
        }
    }

    pub fn get_point_x(&self) -> Vec<u8> {
        assert_eq!(self._public.len(), UNCOMPRESSED_PUBLIC_KEY_SIZE);
        assert_eq!(self._public[0], 0x04);
        let n: &[u8] = &self.get_public_key()[1..];
        let s: &[u8] = &n[0..32];
        s.to_vec()
    }

    pub fn get_point_y(&self) -> Vec<u8> {
        assert_eq!(self._public.len(), UNCOMPRESSED_PUBLIC_KEY_SIZE);
        assert_eq!(self._public[0], 0x04);
        let n: &[u8] = &self.get_public_key()[1..];
        let s: &[u8] = &n[32..];
        s.to_vec()
    }

    pub fn to_jwk(&self) -> KeyPairSecp256K1 {
        let jwk: KeyPairSecp256K1 = KeyPairSecp256K1 {
            kty: "EC".to_string(),
            crv: "secp256k1".to_string(),
            x: base64_url::encode(&self.get_point_x()[..]),
            y: base64_url::encode(&self.get_point_y()[..]),
        };
        jwk
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self._public.clone()
    }

    pub fn get_private_key(&self) -> Vec<u8> {
        self._private.clone()
    }

    pub fn to_public_key(&self, key_id: String, purpose: Vec<String>) -> PublicKeyPayload {
        PublicKeyPayload {
            id: key_id,
            type_field: "EcdsaSecp256k1VerificationKey2019".to_string(),
            jwk: self.to_jwk(),
            purpose,
        }
    }

    pub fn to_hex_key_pair(&self) -> Secp256k1HexKeyPair {
        Secp256k1HexKeyPair {
            public: hex::encode(self.get_public_key()),
            private: hex::encode(self.get_private_key()),
        }
    }

    fn transform_uncompressed_public_key(compressed: Vec<u8>) -> Vec<u8> {
        crate::runtime::secp256k1::Secp256k1::public_key_convert(compressed, false)
    }
}
