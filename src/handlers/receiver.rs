use std::sync::{Arc, atomic::AtomicBool};
use serde::{Serialize, Deserialize};
use rumqttc::{EventLoop, Event, Packet};
use tokio::sync::RwLock;
use std::collections::HashMap;

pub async fn handler(shutdown_marker: Arc<AtomicBool>, mut eventloop: EventLoop, db: Arc<RwLock<HashMap::<String, bool>>>) {
    #[derive(Debug, Serialize, Deserialize)]
    struct Response {
        received_id: String,
    }

    log::info!("start receiver");

    while let Ok(notification) = eventloop.poll().await {
        if shutdown_marker.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        match notification {
            Event::Incoming(v) => {
                if let Packet::Publish(v) = v {
                    if let Ok(payload) = serde_json::from_slice::<Response>(&v.payload) {
                        let mut keys = Vec::<String>::new();
                    
                        db.read().await.keys().enumerate().for_each(|v| {
                            keys.push(v.1.to_string());
                        });
                    
                        let item = keys.iter().find(|v| v.to_string() == payload.received_id);
                    
                        if let Some(v) = item {
                            let _ = db.write().await.insert(v.to_string(), true);
                        }
                    };
                }
            },
            Event::Outgoing(_) => {}
        }
    }

    log::info!("stop receiver");
}

