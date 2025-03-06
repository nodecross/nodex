use crate::controllers::public::nodex_receive;
use cli::AgentCommands;
use dotenvy::dotenv;
use mac_address::get_mac_address;
use nodex::utils::UnwrapLog;
use services::metrics::{MetricsInMemoryCacheService, MetricsWatchService};
use services::nodex::NodeX;
use services::studio::Studio;
use std::env;
use std::fs;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use usecase::metric_usecase::MetricUsecase;
pub mod cli;
mod config;
mod controllers;
mod network;
mod nodex;
mod repository;
mod server;
mod services;
mod usecase;
pub use crate::config::app_config;
pub use crate::config::server_config;
pub use crate::network::network_config;
use protocol::did_webvh::domain::did::Did;

#[tokio::main]
pub async fn run(controlled: bool, options: &cli::AgentOptions) -> std::io::Result<()> {
    dotenv().ok();

    #[cfg(windows)]
    server::windows::kill_other_self_process();

    {
        let config = app_config();
        let config = config.lock();
        config.write().unwrap_log();
    }

    let home_dir = dirs::home_dir().unwrap();
    let config_dir = home_dir.join(".nodex");
    let logs_dir = config_dir.clone().join("logs");

    fs::create_dir_all(&logs_dir).unwrap_log();

    // NOTE: generate Key Chain
    let node_x = NodeX::new();
    let device_did = node_x.create_identifier().await.unwrap();

    if options.config {
        use_cli(options.command.as_ref(), device_did.id.clone());
        return Ok(());
    }

    studio_initialize(device_did.id.clone()).await;
    send_device_info().await;

    let shutdown_token = CancellationToken::new();
    let mut tasks = JoinSet::new();

    let cache_repository =
        MetricsInMemoryCacheService::new(app_config().lock().get_metric_cache_capacity());
    let cache_repository_cloned = cache_repository.clone();
    let shutdown_token_cloned = shutdown_token.clone();
    tasks.spawn(async move {
        let mut metric_usecase = MetricUsecase::new(
            Studio::new(),
            MetricsWatchService::new(),
            app_config(),
            cache_repository_cloned,
            shutdown_token_cloned,
        );
        metric_usecase.collect_task().await
    });
    let shutdown_token_cloned = shutdown_token.clone();
    tasks.spawn(async move {
        let mut metric_usecase = MetricUsecase::new(
            Studio::new(),
            MetricsWatchService::new(),
            app_config(),
            cache_repository,
            shutdown_token_cloned,
        );
        metric_usecase.send_task().await
    });
    tasks.spawn(nodex_receive::polling_task(shutdown_token.clone()));

    // NOTE: booting...
    #[cfg(unix)]
    {
        let runtime_dir = config_dir.clone().join("run");
        fs::create_dir_all(&runtime_dir).unwrap_log();
        let nodex_path = runtime_dir.clone().join("nodex.sock");
        let listener = if !controlled {
            controller::unix_utils::remove_file_if_exists(&nodex_path);
            tokio::net::UnixListener::bind(&nodex_path)?
        } else {
            server::unix::recieve_listener(&nodex_path)?
        };
        let fd = std::os::unix::io::AsRawFd::as_raw_fd(&listener);
        let server = server::unix::make_uds_server(server::make_router(), listener);
        let server =
            server::unix::wrap_with_signal_handler(server, shutdown_token, fd, &nodex_path);
        let (server, _) = tokio::join!(server.join_all(), tasks.join_all());
        server.into_iter().collect::<Result<Vec<()>, _>>()?;
    };

    #[cfg(windows)]
    {
        let port_str =
            env::var("NODEX_SERVER_PORT").expect("NODEX_SERVER_PORT must be set and valid.");
        let port = server::windows::validate_port(&port_str).expect("Invalid port number.");
        let router = server::make_router();
        let server = server::windows::new_web_server(port, router).await?;
        let _ = tokio::join!(server, tasks.join_all());
    };
    Ok(())
}

fn use_cli(command: Option<&AgentCommands>, did: Did) {
    let network_config = crate::network_config();
    let mut network_config = network_config.lock();
    const SECRET_KEY: &str = "secret_key";
    const PROJECT_DID: &str = "project_did";

    if let Some(command) = command {
        match command {
            AgentCommands::Did {} => {
                println!("Node ID: {}", did);
            }
            AgentCommands::Network { command } => match command {
                cli::NetworkSubCommands::Set { key, value } => match key.as_str() {
                    SECRET_KEY => {
                        network_config.save_secret_key(value);
                        log::info!("Network {} is set", SECRET_KEY);
                    }
                    PROJECT_DID => {
                        network_config.save_project_did(value);
                        log::info!("Network {} is set", PROJECT_DID);
                    }
                    _ => {
                        log::info!("key is not found");
                    }
                },
                cli::NetworkSubCommands::Get { key } => match key.as_str() {
                    SECRET_KEY => {
                        if let Some(v) = network_config.get_secret_key() {
                            println!("Network {}: {}", SECRET_KEY, v);
                            return;
                        };
                        log::info!("Network {} is not set", SECRET_KEY);
                    }
                    PROJECT_DID => {
                        if let Some(v) = network_config.get_project_did() {
                            log::info!("Network {}: {}", PROJECT_DID, v);
                            return;
                        };
                        log::info!("Network {} is not set", PROJECT_DID);
                    }
                    _ => {
                        log::info!("key is not found");
                    }
                },
            },
        }
    }
}

async fn studio_initialize(my_did: Did) {
    let project_did = {
        let network = network_config();
        let network_config = network.lock();

        // NOTE: check network secret_key and project_did
        network_config
            .get_secret_key()
            .ok_or("Network secret_key is not set. Please set secret_key use cli")
            .unwrap_log();
        network_config
            .get_project_did()
            .expect("Network project_did is not set. Please set project_did use cli")
    };

    let studio = Studio::new();
    studio
        .register_device(my_did.to_string(), project_did)
        .await
        .unwrap_log();
}

async fn send_device_info() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const OS: &str = env::consts::OS;
    let mac_address: String = match get_mac_address() {
        Ok(Some(ma)) => ma.to_string(),
        _ => String::from("No MAC address found."),
    };

    let project_did = network_config()
        .lock()
        .get_project_did()
        .expect("Failed to get project_did");

    let studio = Studio::new();
    studio
        .send_device_info(
            project_did,
            mac_address,
            VERSION.to_string(),
            OS.to_string(),
        )
        .await
        .unwrap_log();
}
