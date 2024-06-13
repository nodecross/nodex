use http_body_util::BodyExt;
use hyper::{
    body::{Body, Incoming},
    Uri as HyperUri,
    Response,
};
use hyper_util::client::legacy::Client as LegacyClient;
use serde_json::Value;
use std::boxed::Box;
use std::error::Error as StdError;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt as _;

#[cfg(unix)]
use hyperlocal::{UnixClientExt, UnixConnector, Uri as HyperLocalUri};

#[cfg(windows)]
use hyper_util::client::legacy::http::HttpConnector;

pub enum GenericUri {
    #[cfg(unix)]
    Unix(HyperLocalUri),
    #[cfg(windows)]
    Http(HyperUri),
}

impl GenericUri {
    #[cfg(unix)]
    pub fn new_unix(socket_path: PathBuf, path: &str) -> Self {
        Self::Unix(HyperLocalUri::new(socket_path, path))
    }

    #[cfg(windows)]
    pub fn new_http(base_url: &str, path: &str) -> Self {
        let full_url = format!("{}{}", base_url, path);
        Self::Http(full_url.parse().expect("Failed to parse URL"))
    }
}

#[cfg(windows)]
pub fn new_client<B>() -> LegacyClient<HttpConnector, B>
where
    B: Body + Send + 'static + Unpin,
    B::Data: Send,
    B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    LegacyClient::<HttpConnector, B>::new()
}

#[cfg(unix)]
pub fn new_client<B>() -> LegacyClient<UnixConnector, B>
where
    B: Body + Send + 'static + Unpin,
    B::Data: Send,
    B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    LegacyClient::<UnixConnector, B>::unix()
}

pub fn new_uri(url: &str) -> hyper::Uri {
    #[cfg(unix)]
    {
        let homedir = dirs::home_dir().expect("Home directory not found");
        let socket_path = homedir.join(".nodex/run/nodex.sock");
        let generic_uri = GenericUri::new_unix(socket_path, url);
        match generic_uri {
            GenericUri::Unix(uri) => uri.into(),
            _ => panic!("Invalid URI type"),
        }
    }
    #[cfg(windows)]
    {
        let generic_uri = GenericUri::new_http("http://localhost:3000", url);
        match generic_uri {
            GenericUri::Http(uri) => uri.into(),
            _ => panic!("Invalid URI type"),
        }
    }
}


pub async fn response_to_string(mut response: Response<Incoming>) -> anyhow::Result<String> {
    let mut body: Vec<u8> = Vec::with_capacity(2048);

    while let Some(frame_result) = response.frame().await {
        let frame = frame_result?;

        if let Some(segment) = frame.data_ref() {
            body.write_all(segment.iter().as_slice()).await?;
        }
    }

    Ok(String::from_utf8(body)?)
}
