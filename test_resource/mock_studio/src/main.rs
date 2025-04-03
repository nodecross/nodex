mod handler;
mod model;

use anyhow::Result;
use log::info;
use model::AppState;
use std::env;
use std::sync::Arc;
use tokio::signal;
use wiremock::MockServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting wiremock server...");

    // Create shared application state
    let state = Arc::new(AppState::new());
    state.append_project_did();

    let addr = format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or_else(|_| "8020".to_string())
    );

    // Start the mock server
    let listener = std::net::TcpListener::bind(addr).unwrap();
    let server = MockServer::builder().listener(listener).start().await;
    let server_addr = server.address();
    info!("Mock server started at: http://{}", server_addr);

    // Setup all endpoints
    info!("Setting up endpoints...");

    // Basic health endpoint
    handler::setup_health_endpoint(&server).await?;

    // DID document endpoints
    handler::setup_did_registration_endpoint(&server, Arc::clone(&state)).await?;
    handler::setup_did_retrieval_endpoint(&server, Arc::clone(&state)).await?;

    // Device related endpoints
    handler::setup_device_registration_endpoint(&server, Arc::clone(&state)).await?;
    handler::setup_device_info_endpoint(&server).await?;

    // Message activity endpoints
    handler::setup_message_post_endpoint(&server).await?;
    handler::setup_message_put_endpoint(&server).await?;

    // Metrics endpoint
    handler::setup_metrics_endpoint(&server).await?;

    info!("All endpoints are configured and ready");

    // Print server information
    println!("\n=== Mock Server Information ===");
    println!("Server URL: http://{}", server_addr);
    println!("Available endpoints:");
    println!("  - GET    /health");
    println!("  - POST   /webvh/v1/{{uuid}}/did.jsonl");
    println!("  - GET    /webvh/v1/{{uuid}}/did.jsonl");
    println!("  - POST   /v1/device");
    println!("  - POST   /v1/device-info");
    println!("  - POST   /v1/message-activity");
    println!("  - PUT    /v1/message-activity");
    println!("  - POST   /v1/metrics");
    println!("===============================\n");

    // Wait for termination signal instead of stdin
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received termination signal, shutting down...");
        }
        Err(err) => {
            eprintln!("Error waiting for shutdown signal: {}", err);
        }
    }

    info!("Server has been shut down");

    // Server will be shut down when it goes out of scope
    Ok(())
}
