use alloc::vec::Vec;
use picorand::{WyRand, RNG, PicoRandGenerate};
use crate::MUTEX_HANDLERS;
use crate::ffi::Ffi;
use crate::unid::errors::UNiDError;

pub struct Random {}

impl Random {
    #[allow(dead_code)]
    pub fn bytes(length: &usize) -> Vec<u8> {
        let mut rng = RNG::<WyRand, u8>::new(0xDEADBEEF);
        let mut result: Vec<u8> = Vec::new();

        (0..*length).collect::<Vec<usize>>().iter().for_each(|_|
            result.push(rng.generate())
        );

        result
    }

    pub unsafe fn trng_bytes(length: &usize) -> Result<Vec<u8>, UNiDError> {
        let handler = MUTEX_HANDLERS.lock().get_crypto_trng();

        if let Some(..) = handler {
            let random = handler.unwrap()(*length as u32);

            let output = match Ffi::binary_from_ptr(random) {
                Ok(v) => v,
                Err(_) => Vec::from([0])
            };

            Ok(output)
        } else {
            Err(UNiDError{})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let result = Random::bytes(&32);

        assert_eq!(result.len(), 32);
    }
}