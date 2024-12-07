use reqwest::Client;
use std::time::Duration;

pub async fn can_connect_to_download_server(url: &str) -> bool {
    let client = Client::builder().timeout(Duration::from_secs(5)).build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
