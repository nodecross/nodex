use protocol::keyring::keypair::KeyPairing;

pub trait DidAccessor {
    fn get_my_did(&self) -> String;
    fn get_my_keyring(&self) -> KeyPairing;
}

pub struct DidAccessorImpl {}

impl DidAccessor for DidAccessorImpl {
    fn get_my_did(&self) -> String {
        let config = crate::app_config();
        let config = config.lock();
        config.get_did().unwrap().to_string()
    }

    fn get_my_keyring(&self) -> KeyPairing {
        let config = crate::app_config();
        let config = config.lock();
        config.load_keyring().expect("failed to load keyring")
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct MockDidAccessor {
        my_did: String,
        my_keyring: KeyPairing,
    }

    impl MockDidAccessor {
        pub fn new(my_did: String, my_keyring: KeyPairing) -> MockDidAccessor {
            MockDidAccessor { my_did, my_keyring }
        }
    }

    impl DidAccessor for MockDidAccessor {
        fn get_my_did(&self) -> String {
            self.my_did.clone()
        }

        fn get_my_keyring(&self) -> KeyPairing {
            self.my_keyring.clone()
        }
    }
}
