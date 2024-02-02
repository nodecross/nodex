use chrono::Utc;

use crate::{
    controllers::public::nodex_receive::ConnectionRepository, services::hub::Hub,
};
use std::sync::{atomic::AtomicBool, Arc};

pub async fn handler(
    shutdown_marker: Arc<AtomicBool>,
    connection_repository: ConnectionRepository,
) {
    let (heartbeat_interval_sec, project_did) = {
        let network = crate::network_config();
        let network = network.lock();

        let heartbeat_interval_sec = match network.get_heartbeat() {
            Some(sec) => sec,
            None => {
                log::info!("heartbeat is disabled");
                return;
            }
        };

        let project_did = network.get_project_did().expect("project_did is not set");

        (heartbeat_interval_sec, project_did)
    };

    log::info!("heartbeat task is started");
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(heartbeat_interval_sec));
    let hub = Hub::new();

    while !shutdown_marker.load(std::sync::atomic::Ordering::SeqCst) {
        interval.tick().await;

        let is_active = connection_repository.connection_count() > 0;
        if let Err(e) = hub.heartbeat(&project_did, is_active, Utc::now()).await {
            log::error!("failed to send heartbeat: {:?}", e);
        }
    }

    log::info!("heartbeat task is stopped");
}
