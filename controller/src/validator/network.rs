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

#[cfg(test)]
mod tests {
    use super::can_connect_to_download_server;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_can_connect_to_download_server() {
        let server = MockServer::start();

        let success_mock = server.mock(|when, then| {
            when.method("GET").path("/success");
            then.status(200).body("Success");
        });

        let failure_mock = server.mock(|when, then| {
            when.method("GET").path("/failure");
            then.status(500).body("Internal Server Error");
        });

        let success_url = format!("{}{}", server.base_url(), "/success");
        let result = can_connect_to_download_server(&success_url).await;
        assert!(result, "Expected success for the valid URL");
        success_mock.assert();

        let failure_url = format!("{}{}", server.base_url(), "/failure");
        let result = can_connect_to_download_server(&failure_url).await;
        assert!(!result, "Expected failure for the invalid URL");
        failure_mock.assert();

        let invalid_url = "http://invalid-url";
        let result = can_connect_to_download_server(invalid_url).await;
        assert!(!result, "Expected failure for an unreachable URL");
    }
}
