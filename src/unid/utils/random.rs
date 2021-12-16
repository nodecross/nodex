use alloc::vec::Vec;
use picorand::{WyRand, RNG, PicoRandGenerate};

pub struct Random {}

impl Random {
    pub fn bytes(length: &usize) -> Vec<u8> {
        let mut rng = RNG::<WyRand, u8>::new(0xDEADBEEF);
        let mut result: Vec<u8> = Vec::new();

        (0..*length).collect::<Vec<usize>>().iter().for_each(|_|
            result.push(rng.generate())
        );

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_hash() {
        let result = Random::bytes(&32);

        assert_eq!(result, Vec::from([0]));
        assert_eq!(result.len(), 32);
    }
}