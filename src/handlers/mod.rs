use crate::nodex::errors::NodeXError;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::oneshot;

pub mod heartbeat;
pub mod receiver;
pub mod sender;

type Responder = oneshot::Sender<bool>;

#[derive(Debug)]
pub enum Command {
    Send { value: Value, resp: Responder },
}

#[async_trait]
pub trait TransferClient: Send + Sync {
    async fn send(&self, value: Value) -> Result<bool, NodeXError>;
}

pub struct MqttClient {
    sender: tokio::sync::mpsc::Sender<Command>,
}

impl MqttClient {
    pub fn new(sender: tokio::sync::mpsc::Sender<Command>) -> Self {
        MqttClient { sender }
    }
}

#[async_trait]
impl TransferClient for MqttClient {
    async fn send(&self, value: Value) -> Result<bool, NodeXError> {
        let (tx, rx) = oneshot::channel();

        let command = Command::Send { value, resp: tx };

        self.sender.send(command).await.map_err(|e| {
            log::error!("{:?}", e.to_string());
            NodeXError {}
        })?;

        match rx.await {
            Ok(is_success) => Ok(is_success),
            Err(e) => {
                println!("error!!!!!");
                log::error!("{:?}", e.to_string());
                Err(NodeXError {})
            }
        }
    }
}
