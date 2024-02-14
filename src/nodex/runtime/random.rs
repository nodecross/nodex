use thiserror::Error;

pub struct Random {}

#[derive(Debug, Error)]
pub enum RandomError {
    #[error(transparent)]
    GetRandomError(#[from] getrandom::Error),
}

impl Random {
    pub fn bytes(size: &usize) -> Result<Vec<u8>, RandomError> {
        let mut bytes = vec![0u8; *size];

        getrandom::getrandom(&mut bytes)?;
        Ok(bytes)
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
