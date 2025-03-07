pub mod encrypted;
pub mod sign_encrypt;
pub mod types;

#[cfg(test)]
pub mod test_utils {
    use rand::distributions::{Alphanumeric, DistString as _};

    pub fn create_random_did() -> String {
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        format!("did:nodex:test:{}", random_string)
    }
}
