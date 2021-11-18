pub struct Encoder {}

impl Encoder {
    pub fn encode(content_vec: Vec<u8>) -> String {
        let content_u8: &[u8] = &content_vec[..];
        let content_base64url: String = base64_url::encode(content_u8);

        content_base64url
    }
}