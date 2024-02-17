use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::oneshot;

pub mod sender;

type Responder = oneshot::Sender<bool>;

#[derive(Debug)]
pub enum Command {
    Send { value: Value, resp: Responder },
}

#[async_trait]
pub trait TransferClient: Send + Sync {
    async fn send(&self, value: Value) -> anyhow::Result<bool>;
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
    async fn send(&self, value: Value) -> anyhow::Result<bool> {
        let (tx, rx) = oneshot::channel();

        let command = Command::Send { value, resp: tx };

        self.sender.send(command).await?;

        Ok(rx.await?)
    }
}
