pub mod attribute_usecase;
pub mod custom_metric_usecase;
pub mod didcomm_message_usecase;
pub mod event_usecase;
pub mod metric_usecase;

#[cfg(test)]
mod test_util {
    use protocol::keyring::keypair::KeyPairing;
    use protocol::rand_core::OsRng;

    #[derive(Clone)]
    pub struct TestPresets {
        pub from_did: String,
        pub from_keyring: KeyPairing,
        pub to_did: String,
        pub to_keyring: KeyPairing,
    }

    impl Default for TestPresets {
        fn default() -> Self {
            let from_keyring = KeyPairing::create_keyring(OsRng);
            let to_keyring = KeyPairing::create_keyring(OsRng);

            let from_did =
                "did:webvh:QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com:from"
                    .to_string();
            let to_did = "did:webvh:QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr:example.com:to"
                .to_string();

            TestPresets {
                from_did,
                from_keyring,
                to_did,
                to_keyring,
            }
        }
    }
}
