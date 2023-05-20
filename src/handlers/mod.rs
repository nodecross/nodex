use serde_json::Value;
use tokio::sync::oneshot;

pub mod receiver;
pub mod sender;

type Responder = oneshot::Sender<bool>;

#[derive(Debug)]
pub enum Command {
    Send { value: Value, resp: Responder },
}
