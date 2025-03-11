use crate::did_webvh::infra::did_webvh_data_store::DidWebvhDataStore;

#[derive(Clone)]
pub struct DidWebvhServiceImpl<C: DidWebvhDataStore> {
    pub data_store: C,
    pub use_https: bool,
}

impl<C: DidWebvhDataStore> DidWebvhServiceImpl<C> {
    pub fn new(data_store: C, use_https: bool) -> Self {
        Self {
            data_store,
            use_https,
        }
    }
}
