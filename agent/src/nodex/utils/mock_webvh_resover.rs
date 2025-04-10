#[cfg(test)]
pub mod mocks {
    use protocol::rand_core::OsRng;
    use protocol::{
        did_webvh::{
            domain::{
                did::{Did, DidWebvh},
                did_document::DidDocument,
            },
            service::resolver::resolver_service::{
                DidWebvhResolverService, ResolveIdentifierError,
            },
        },
        keyring::keypair::KeyPairing,
    };

    pub struct MockDidWebvhResolverService {
        did_document: DidDocument,
    }

    impl MockDidWebvhResolverService {
        pub fn new(my_did: String, my_keyring: KeyPairing) -> Self {
            let did = my_did.parse::<Did>().unwrap();
            let vms = my_keyring.to_verification_methods(&did).unwrap();
            let mut doc = DidDocument::new(did);
            for vm in vms {
                doc.add_verification_method(vm);
            }
            Self { did_document: doc }
        }

        pub fn empty() -> Self {
            let keyring = KeyPairing::create_keyring(OsRng);
            let random_string = "QmdEjpG2gwEWZAx8YjBrw7mF1iuCqgrMh8S63M7PaC1Ldr";
            let did = DidWebvh::new(random_string, "domain.com:empty").unwrap();
            let did = did.get_did().clone();
            Self::new(did.into_inner(), keyring)
        }
    }

    impl DidWebvhResolverService for MockDidWebvhResolverService {
        type DidWebvhResolverError = ResolveIdentifierError;

        async fn resolve_identifier(
            &mut self,
            did: &Did,
        ) -> Result<Option<DidDocument>, Self::DidWebvhResolverError> {
            if did == &self.did_document.id {
                Ok(Some(self.did_document.clone()))
            } else {
                Ok(None)
            }
        }
    }
}
