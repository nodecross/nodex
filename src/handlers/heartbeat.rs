use crate::{
    controllers::public::nodex_receive::ConnectionRepository, network::Network, services::hub::Hub,
};
use std::sync::{atomic::AtomicBool, Arc};

pub async fn handler(
    shutdown_marker: Arc<AtomicBool>,
    connection_repository: ConnectionRepository,
) {
    let network = Network::new();

    let heartbeat_interval_sec = if let Some(heartbeat_interval_sec) = network.get_heartbeat() {
        heartbeat_interval_sec
    } else {
        log::info!("heartbeat is disabled");
        return;
    };

    let project_did = network.root.project_did.expect("project_did is not set");

    log::info!("heartbeat task is started");

    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(heartbeat_interval_sec));
    let hub = Hub::new();

    while !shutdown_marker.load(std::sync::atomic::Ordering::SeqCst) {
        interval.tick().await;

        let is_active = connection_repository.connection_count() > 0;

        if let Err(e) = hub.heartbeat(&project_did, is_active).await {
            log::error!("failed to send heartbeat: {:?}", e);
        }
    }

    log::info!("heartbeat task is stopped");
}
