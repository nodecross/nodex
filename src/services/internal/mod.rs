pub mod did_vc;
pub mod did_vp;
pub mod didcomm_encrypted;
pub mod didcomm_plaintext;
pub mod didcomm_signed;
pub mod types;
use crate::server_config;

fn attachment_link() -> String {
    let server_config = server_config();
    server_config.did_attachment_link()
}
