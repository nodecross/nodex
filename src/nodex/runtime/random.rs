pub struct Random {}

impl Random {
    pub fn bytes(size: &usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = vec![0u8; *size];

        match getrandom::getrandom(&mut bytes) {
            Ok(_) => Ok(bytes),
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_success_random_bytes_32() {
        let result = match Random::bytes(&32) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result.len(), 32);
    }

    #[test]
    fn it_should_success_random_bytes_128() {
        let result = match Random::bytes(&128) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result.len(), 128);
    }
}
