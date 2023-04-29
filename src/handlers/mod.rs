use tokio::sync::oneshot;
use serde_json::Value;

pub mod sender;
pub mod receiver;

type Responder = oneshot::Sender<bool>;

#[derive(Debug)]
pub enum Command {
    Send {
        value: Value,
        resp: Responder,
    }
}