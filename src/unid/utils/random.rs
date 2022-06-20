use alloc::vec::Vec;

pub struct Random {}

impl Random {
    pub fn bytes(length: &usize) -> Vec<u8> {
        let mut buf = vec![Default::default(), length];
        getrandom::getrandom(&mut buf);
        buf.to_vec()
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