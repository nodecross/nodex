use rumqttc::{AsyncClient, QoS};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc::Receiver, RwLock};
use tokio::time::{sleep, Duration, Instant};

use super::Command;

pub async fn handler(
    mut rx: Receiver<Command>,
    client: AsyncClient,
    db: Arc<RwLock<HashMap<String, bool>>>,
    topic: String,
) {
    log::info!("start sender");

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Send { value, resp } => {
                let id = cuid::cuid2();

                let payload: Value = json!({
                    "id": id,
                    "value": value,
                });

                if (client
                    .publish(
                        topic.to_string(),
                        QoS::AtLeastOnce,
                        false,
                        payload.to_string().as_bytes(),
                    )
                    .await)
                    .is_ok()
                {
                    db.write().await.insert(id.clone(), false);

                    let start = Instant::now();
                    let threshold = Duration::from_secs(15);

                    loop {
                        if threshold < start.elapsed() {
                            _ = resp.send(false);
                            break;
                        }

                        match db.read().await.get(&id) {
                            Some(v) => {
                                if *v {
                                    _ = resp.send(true);
                                    break;
                                }
                            }
                            None => {
                                continue;
                            }
                        }

                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    log::info!("stop sender");
}
