use crate::model::*;
use log::{debug, error, info};
use protocol::did_webvh::domain::did_log_entry::DidLogEntry;
use regex::Regex;
use std::sync::Arc;
use wiremock::matchers::{header, method, path, path_regex};
use wiremock::{Mock, ResponseTemplate};

// Health check endpoint (GET /health)
pub async fn setup_health_endpoint(server: &wiremock::MockServer) -> anyhow::Result<()> {
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "status": "ok" })),
        )
        .mount(server)
        .await;

    debug!("Health endpoint mounted");
    Ok(())
}

// DID registration endpoint (POST /webvh/v1/{uuid}/did.jsonl)
pub async fn setup_did_registration_endpoint(
    server: &wiremock::MockServer,
    state: Arc<AppState>,
) -> anyhow::Result<()> {
    info!("Setting up DID registration endpoint");
    Mock::given(method("POST"))
        .and(path_regex(r"^/webvh/v1/([^/]+)"))
        .and(header("content-type", "application/json"))
        .respond_with(move |request: &wiremock::Request| {
            // Extract UUID from path
            let path = request.url.path();
            let uuid_regex = Regex::new(r"^/webvh/v1/([^/]+)").unwrap();

            let uuid = match uuid_regex.captures(path) {
                Some(caps) => caps.get(1).unwrap().as_str().to_string(),
                None => {
                    error!("Failed to extract UUID from path: {}", path);
                    return ResponseTemplate::new(400).set_body_json(serde_json::json!({
                        "error": "Invalid path format",
                        "code": "bad_request"
                    }));
                }
            };

            debug!("Extracted UUID: {}", uuid);

            // Parse request body
            let body = request.body.as_ref();
            let result: Result<Vec<DidLogEntry>, _> = serde_json::from_slice(body);

            match result {
                Ok(entries) => {
                    // Store in hashmap
                    let state_clone = Arc::clone(&state);
                    if let Ok(mut store) = state_clone.did_store.lock() {
                        store.insert(uuid.clone(), entries.clone());
                    }

                    debug!("DID registration completed: UUID: {}", uuid);

                    let doc = entries.last().unwrap().state.clone();
                    ResponseTemplate::new(200)
                        .insert_header("content-type", "application/json")
                        .set_body_json(&doc)
                }
                Err(e) => {
                    error!("Failed to parse DID document: {}", e);

                    ResponseTemplate::new(400)
                        .insert_header("content-type", "application/json")
                        .set_body_json(serde_json::json!({
                            "error": "Invalid JSON",
                            "code": "bad_request",
                            "message": e.to_string()
                        }))
                }
            }
        })
        .mount(server)
        .await;

    debug!("DID registration endpoint mounted");
    Ok(())
}

// DID retrieval endpoint (GET /webvh/v1/{uuid}/did.jsonl)
pub async fn setup_did_retrieval_endpoint(
    server: &wiremock::MockServer,
    state: Arc<AppState>,
) -> anyhow::Result<()> {
    Mock::given(method("GET"))
        .and(path_regex(r"^/webvh/v1/([^/]+)/did\.jsonl$"))
        .respond_with(move |request: &wiremock::Request| {
            // Extract UUID from path
            let path = request.url.path();
            let uuid_regex = Regex::new(r"^/webvh/v1/([^/]+)/did\.jsonl$").unwrap();

            let uuid = match uuid_regex.captures(path) {
                Some(caps) => caps.get(1).unwrap().as_str().to_string(),
                None => {
                    error!("Failed to extract UUID from path: {}", path);
                    return ResponseTemplate::new(400).set_body_json(serde_json::json!({
                        "error": "Invalid path format",
                        "code": "bad_request"
                    }));
                }
            };

            debug!("DID retrieval request - UUID: {}", uuid);

            // Look up document in store
            let state_clone = Arc::clone(&state);
            let store_result = state_clone.did_store.lock();

            match store_result {
                Ok(store) => match store.get(&uuid) {
                    Some(entries) => {
                        debug!("DID retrieval success - UUID: {}", uuid);

                        ResponseTemplate::new(200)
                            .insert_header("content-type", "application/json")
                            .set_body_json(&entries)
                    }
                    None => {
                        error!("DID not found - UUID: {}", uuid);

                        ResponseTemplate::new(404)
                            .insert_header("content-type", "application/json")
                            .set_body_json(serde_json::json!({
                                "error": "DID not found",
                                "code": "not_found",
                                "uuid": uuid
                            }))
                    }
                },
                Err(e) => {
                    error!("Failed to acquire lock on store: {}", e);

                    ResponseTemplate::new(500)
                        .insert_header("content-type", "application/json")
                        .set_body_json(serde_json::json!({
                            "error": "Internal server error",
                            "code": "internal_error"
                        }))
                }
            }
        })
        .mount(server)
        .await;

    debug!("DID retrieval endpoint mounted");
    Ok(())
}

// Device registration endpoint (POST /v1/device)
pub async fn setup_device_registration_endpoint(
    server: &wiremock::MockServer,
    state: Arc<AppState>,
) -> anyhow::Result<()> {
    Mock::given(method("POST"))
        .and(path("/v1/device"))
        .and(header("content-type", "application/json"))
        .respond_with(move |request: &wiremock::Request| {
            // Parse request body
            let body = request.body.as_ref();
            let result: Result<RegisterDeviceRequest, _> = serde_json::from_slice(body);

            match result {
                Ok(req) => {
                    // Store device registration
                    let state_clone = Arc::clone(&state);
                    if let Ok(mut store) = state_clone.device_store.lock() {
                        store.insert(req.device_did.clone(), req.project_did.clone());
                    }

                    debug!(
                        "Registered device: {} for project: {}",
                        req.device_did, req.project_did
                    );

                    ResponseTemplate::new(200)
                        .insert_header("content-type", "application/json")
                        .set_body_json(serde_json::json!({}))
                }
                Err(e) => {
                    error!("Failed to parse device registration request: {}", e);

                    ResponseTemplate::new(400)
                        .insert_header("content-type", "application/json")
                        .set_body_json(serde_json::json!({
                            "error": "Invalid JSON",
                            "code": "bad_request",
                            "message": e.to_string()
                        }))
                }
            }
        })
        .mount(server)
        .await;

    debug!("Device registration endpoint mounted");
    Ok(())
}

// Device info endpoint (POST /v1/device-info)
pub async fn setup_device_info_endpoint(server: &wiremock::MockServer) -> anyhow::Result<()> {
    Mock::given(method("POST"))
        .and(path("/v1/device-info"))
        .respond_with(move |request: &wiremock::Request| {
            // Log content type and body size
            let content_type = request
                .headers
                .get("content-type")
                .map(|v| v.to_str().unwrap_or("application/cbor"))
                .unwrap_or("application/cbor");

            let body_size = request.body.len();

            debug!(
                "Received device info with Content-Type: {}, size: {} bytes",
                content_type, body_size
            );

            // For mock server, we don't need to actually decode CBOR/COSE
            // Just acknowledge receipt
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_json(serde_json::json!({}))
        })
        .mount(server)
        .await;

    debug!("Device info endpoint mounted");
    Ok(())
}

// Message activity endpoint (POST /v1/message-activity)
pub async fn setup_message_post_endpoint(server: &wiremock::MockServer) -> anyhow::Result<()> {
    Mock::given(method("POST"))
        .and(path("/v1/message-activity"))
        .and(header("content-type", "application/json"))
        .respond_with(move |request: &wiremock::Request| {
            // Parse request body as DidCommMessage
            let body = request.body.as_ref();
            let result: Result<DidCommMessage, _> = serde_json::from_slice(body);

            match result {
                Ok(msg) => {
                    debug!(
                        "Received message activity (POST): {}",
                        msg.ciphertext.unwrap_or_default()
                    );

                    ResponseTemplate::new(200)
                        .insert_header("content-type", "application/json")
                        .set_body_json(serde_json::json!({}))
                }
                Err(e) => {
                    error!("Failed to parse message activity: {}", e);

                    ResponseTemplate::new(400)
                        .insert_header("content-type", "application/json")
                        .set_body_json(serde_json::json!({
                            "error": "Invalid JSON",
                            "code": "bad_request",
                            "message": e.to_string()
                        }))
                }
            }
        })
        .mount(server)
        .await;

    debug!("Message activity POST endpoint mounted");
    Ok(())
}

// Message activity endpoint (PUT /v1/message-activity)
pub async fn setup_message_put_endpoint(server: &wiremock::MockServer) -> anyhow::Result<()> {
    Mock::given(method("PUT"))
        .and(path("/v1/message-activity"))
        .and(header("content-type", "application/json"))
        .respond_with(move |request: &wiremock::Request| {
            // Parse request body as DidCommMessage
            let body = request.body.as_ref();
            let result: Result<DidCommMessage, _> = serde_json::from_slice(body);

            match result {
                Ok(msg) => {
                    debug!(
                        "Received message activity (PUT): {}",
                        msg.ciphertext.unwrap_or_default()
                    );

                    ResponseTemplate::new(200)
                        .insert_header("content-type", "application/json")
                        .set_body_json(serde_json::json!({}))
                }
                Err(e) => {
                    error!("Failed to parse message activity: {}", e);

                    ResponseTemplate::new(400)
                        .insert_header("content-type", "application/json")
                        .set_body_json(serde_json::json!({
                            "error": "Invalid JSON",
                            "code": "bad_request",
                            "message": e.to_string()
                        }))
                }
            }
        })
        .mount(server)
        .await;

    debug!("Message activity PUT endpoint mounted");
    Ok(())
}

// Metrics endpoint (POST /v1/metrics)
pub async fn setup_metrics_endpoint(server: &wiremock::MockServer) -> anyhow::Result<()> {
    Mock::given(method("POST"))
        .and(path("/v1/metrics"))
        .respond_with(move |request: &wiremock::Request| {
            // Log content type and body size
            let content_type = request
                .headers
                .get("content-type")
                .map(|v| v.to_str().unwrap_or("application/cbor"))
                .unwrap_or("application/cbor");

            let body_size = request.body.len();

            debug!(
                "Received metrics with Content-Type: {}, size: {} bytes",
                content_type, body_size
            );

            // For mock server, we don't need to actually decode CBOR/COSE
            // Just acknowledge receipt
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_json(serde_json::json!({}))
        })
        .mount(server)
        .await;

    debug!("Metrics endpoint mounted");
    Ok(())
}
