use base64::{engine::general_purpose::STANDARD as BASE64_STD_ENGINE, Engine as _};
use sigstore::{
    bundle::verify::{policy, VerificationPolicy},
    cosign::{
        bundle::SignedArtifactBundle,
        {client::Client, CosignCapabilities},
    },
    errors::SigstoreError,
};
use std::path::Path;
use x509_cert;

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("failed to parse signed artifact bundle JSON: {0}")]
    BundleParse(serde_json::Error),
    #[error("failed to decode base64 certificate: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("failed to convert decoded certificate to UTF-8 string: {0}")]
    Utf8Conversion(#[from] std::string::FromUtf8Error),
    #[error("failed to load X.509 certificate chain from PEM data: {0}")]
    X509CertLoad(#[from] x509_cert::der::Error),
    #[error("failed to read file: {0}")]
    FileRead(#[source] std::io::Error),
    #[error("failed to verify blob: {0}")]
    BlobVerification(#[source] SigstoreError),
}

pub trait Verifier: Send + Sync {
    fn verify(
        &self,
        bundle_path: &Path,
        blob_path: &Path,
        identity: &str,
        issuer: &str,
    ) -> Result<(), VerifyError>;

    fn decode_cert(&self, cert: &str) -> Result<String, VerifyError>;
}

pub struct BundleVerifier;

impl Verifier for BundleVerifier {
    fn verify(
        &self,
        bundle_path: &Path,
        blob_path: &Path,
        identity: &str,
        issuer: &str,
    ) -> Result<(), VerifyError> {
        let blob = std::fs::read(blob_path).map_err(VerifyError::FileRead)?;
        let bundle_json = std::fs::read_to_string(bundle_path).map_err(VerifyError::FileRead)?;
        let bundle: SignedArtifactBundle =
            serde_json::from_str(&bundle_json).map_err(VerifyError::BundleParse)?;

        let decoded_cert = self.decode_cert(bundle.cert.as_str())?;
        let cert_chain = x509_cert::Certificate::load_pem_chain(decoded_cert.as_bytes())
            .map_err(VerifyError::X509CertLoad)?;

        let id_policy = policy::Identity::new(identity, issuer);
        id_policy.verify(&cert_chain[0]).expect("Failed to verify");

        Client::verify_blob(&decoded_cert, bundle.base64_signature.trim(), &blob)
            .map_err(VerifyError::BlobVerification)
    }

    fn decode_cert(&self, cert: &str) -> Result<String, VerifyError> {
        let cert_data = BASE64_STD_ENGINE.decode(cert)?;
        let cert_str = String::from_utf8(cert_data)?;
        Ok(cert_str)
    }
}
