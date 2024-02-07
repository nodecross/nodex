use hmac::{Hmac, Mac};

use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub trait ProjectVerifier {
    fn create_project_hmac(&self) -> anyhow::Result<String>;
    fn verify_project_hmac(&self, signature: &str) -> anyhow::Result<bool>;
}

pub struct ProjectVerifierImplOnNetworkConfig {}

impl ProjectVerifierImplOnNetworkConfig {
    pub fn new() -> Self {
        ProjectVerifierImplOnNetworkConfig {}
    }
}

impl ProjectVerifier for ProjectVerifierImplOnNetworkConfig {
    fn create_project_hmac(&self) -> anyhow::Result<String> {
        let network = crate::network_config();
        let network = network.lock();
        let project_did = network
            .get_project_did()
            .ok_or(anyhow::anyhow!("project_did is not set"))?;
        let secret_key = network
            .get_secret_key()
            .ok_or(anyhow::anyhow!("secret key is not set"))?;

        let mut hmac = HmacSha256::new_from_slice(secret_key.as_bytes())?;
        hmac.update(project_did.as_bytes());

        let signature = hex::encode(hmac.finalize().into_bytes());

        Ok(signature)
    }

    fn verify_project_hmac(&self, signature: &str) -> anyhow::Result<bool> {
        let network = crate::network_config();
        let network = network.lock();
        let project_did = network
            .get_project_did()
            .ok_or(anyhow::anyhow!("project_did is not set"))?;
        let secret_key = network
            .get_secret_key()
            .ok_or(anyhow::anyhow!("secret key is not set"))?;

        let mut hmac = HmacSha256::new_from_slice(secret_key.as_bytes())?;
        hmac.update(project_did.as_bytes());

        Ok(hmac.verify_slice(signature.as_bytes()).is_ok())
    }
}
