use hyper::{Method, Request, StatusCode};
use serde_json::json;
use std::fs::read;

use e2e::common::platform_client::{new_client, new_uri, response_to_string};

async fn create_didcomm_message_scenario() -> anyhow::Result<String> {
    let homedir = dirs::home_dir().unwrap();
    let client = new_client();

    let my_did = {
        let config = read(homedir.join(".config/nodex/config.json"))?;
        let config = serde_json::from_slice::<serde_json::Value>(&config)?;
        config["did"].as_str().unwrap().to_string()
    };
    let body = json!({
        "destination_did": my_did,
        "operation_tag": "test",
        "message": "Hello, world!"
    })
    .to_string();
    let create_url = new_uri("/create-didcomm-message");
    let request = Request::builder()
        .method(Method::POST)
        .uri(create_url)
        .header("Content-Type", "application/json")
        .body(body)?;

    let response = client.request(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let body: String = response_to_string(response).await?;
    // parse check
    let _parsed = serde_json::from_str::<serde_json::Value>(&body)?;

    Ok(body)
}

async fn verify_didcomm_message_scenario(input: String) -> anyhow::Result<()> {
    let client = new_client();

    let body = json!({
        "message": input
    })
    .to_string();
    let verify_url = new_uri("/verify-didcomm-message");
    let request = Request::builder()
        .method(Method::POST)
        .uri(verify_url)
        .header("Content-Type", "application/json")
        .body(body)?;

    let response = client.request(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_to_string(response).await?;
    let body: String = serde_json::from_str(&body)?;
    let body_json = serde_json::from_str::<serde_json::Value>(&body)?;

    assert_eq!(
        body_json
            .get("payload")
            .and_then(|v| v.as_str())
            .expect("payload not found"),
        "Hello, world!"
    );

    Ok(())
}

#[tokio::test]
async fn test() {
    let input = create_didcomm_message_scenario()
        .await
        .expect("failed to create_didcomm_message_scenario");
    verify_didcomm_message_scenario(input)
        .await
        .expect("failed to verify_didcomm_message_scenario");
}
