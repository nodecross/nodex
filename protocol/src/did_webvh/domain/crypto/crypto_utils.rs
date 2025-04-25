use bs58;
use ed25519_dalek::*;
use multibase::Base;
use multihash::Multihash;
use sha2::{Digest, Sha256};

const SHA256: u64 = 0x12;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("Failed to generate hash")]
    GenerateHash,
    #[error("Failed to sign data: {0}")]
    SignData(String),
    #[error("Failed to verify signature: {0}")]
    Signature(String),
}

pub fn generate_multihash_with_base58_encode(data: &[u8]) -> Result<String, CryptoError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let wrapped_hash =
        Multihash::<32>::wrap(SHA256, &hash).map_err(|_| CryptoError::GenerateHash)?;
    Ok(bs58::encode(wrapped_hash.to_bytes()).into_string())
}

pub fn validate_hash(hash: &str) -> bool {
    match bs58::decode(hash).into_vec() {
        Ok(decoded) => match Multihash::<32>::from_bytes(&decoded) {
            Ok(multihash) => multihash.code() == SHA256,
            Err(_) => false,
        },
        Err(e) => {
            println!("Error decoding hash: {:?}", e);
            false
        }
    }
}

pub fn sign_data(data: &[u8], key: &[u8]) -> Result<String, CryptoError> {
    let sign_key = SigningKey::from_bytes(
        key.try_into()
            .map_err(|e| CryptoError::SignData(format!("{:?}", e)))?,
    );
    let signature = sign_key.sign(data);
    let proof_value = multibase_encode(&signature.to_bytes());
    Ok(proof_value)
}

pub fn verify_signature(data: &[u8], signature: &[u8], key: &[u8]) -> Result<bool, CryptoError> {
    let verify_key = VerifyingKey::from_bytes(
        key.try_into()
            .map_err(|_| CryptoError::Signature("Failed to convert key to 32bytes".to_string()))?,
    )
    .map_err(|e| CryptoError::Signature(e.to_string()))?;
    let signature =
        Signature::from_bytes(signature.try_into().expect("Failed to convert signature"));
    let result = verify_key.verify(data, &signature);
    Ok(result.is_ok())
}

pub fn multibase_encode(data: &[u8]) -> String {
    multibase::encode(Base::Base58Btc, data)
}

pub fn multibase_decode(data: &str) -> Result<Vec<u8>, multibase::Error> {
    let (_, decoded) = multibase::decode(data)?;
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyring::keypair::*;
    use crate::rand_core::OsRng;

    #[test]
    fn test_generate_multihash_with_base58encode() {
        let data = "z6MkoSFjacZb7R82htx8n1AkpgLQWR7CA6rigsc2VH9acLuF";
        let hash = generate_multihash_with_base58_encode(data.as_bytes()).unwrap();
        assert_eq!(hash, "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr");
    }

    #[test]
    fn test_validate_hash() {
        let encoded_hash = "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr";
        assert!(validate_hash(encoded_hash));

        let invalid_hash = "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1L";
        assert!(!validate_hash(invalid_hash));

        let invalid_hash = "scid";
        assert!(!validate_hash(invalid_hash));
    }

    #[test]
    fn test_sign_data() -> Result<(), CryptoError> {
        let plaintext = "Hello, World!";
        let keypairs = KeyPairing::create_keyring(OsRng);
        let sec_key = keypairs.didwebvh_update.get_secret_key().to_bytes();
        let pub_key = keypairs.didwebvh_update.get_public_key().to_bytes();
        let signature = sign_data(plaintext.as_bytes(), &sec_key).unwrap();
        let decoded_signature = multibase_decode(&signature).unwrap();
        let verified = verify_signature(plaintext.as_bytes(), &decoded_signature, &pub_key)?;
        assert!(verified);
        Ok(())
    }

    #[test]
    fn test_multibase_encode() {
        let data = "Hello, World!";
        let encoded = multibase_encode(data.as_bytes());
        assert_eq!(encoded, "z72k1xXWG59fYdzSNoA");
    }

    #[test]
    fn test_multibase_decode() {
        let encoded = "z72k1xXWG59fYdzSNoA";
        let decoded = multibase_decode(encoded).unwrap();
        assert_eq!(decoded, "Hello, World!".as_bytes());
    }

    #[test]
    fn test_keyring_multibase() {
        let keypairs = KeyPairing::create_keyring(OsRng);
        let public_key = keypairs.didwebvh_update.get_public_key().to_bytes();
        let encoded = multibase_encode(&public_key);
        let decoded = multibase_decode(&encoded).unwrap();
        assert_eq!(decoded, public_key);
    }
}
