use protocol::did_webvh::domain::did::Did;
use protocol::keyring::keypair::KeyPairing;

pub trait DidAccessor {
    fn get_my_did(&self) -> Did;
    fn get_my_keyring(&self) -> KeyPairing;
}

pub struct DidAccessorImpl {}

impl DidAccessor for DidAccessorImpl {
    fn get_my_did(&self) -> Did {
        let config = crate::app_config();
        let config = config.lock();
        config.get_did().unwrap().parse().unwrap()
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
        my_did: Did,
        my_keyring: KeyPairing,
    }

    impl MockDidAccessor {
        pub fn new(my_did: String, my_keyring: KeyPairing) -> MockDidAccessor {
            MockDidAccessor {
                my_did: my_did.parse().unwrap(),
                my_keyring,
            }
        }
    }

    impl DidAccessor for MockDidAccessor {
        fn get_my_did(&self) -> Did {
            self.my_did.clone()
        }

        fn get_my_keyring(&self) -> KeyPairing {
            self.my_keyring.clone()
        }
    }
}
