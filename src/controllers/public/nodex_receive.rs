use actix::prelude::*;
use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::{atomic::AtomicBool, Arc, RwLock},
    time::{Duration, SystemTime},
};

use crate::{
    nodex::errors::NodeXError,
    server,
    services::{hub::Hub, internal::didcomm_encrypted::DIDCommEncryptedService},
};

#[derive(Debug, Clone)]
pub struct ConnectionRepository {
    connections: Arc<RwLock<HashSet<Addr<MessageReceiveActor>>>>,
}

impl ConnectionRepository {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn insert(&mut self, addr: Addr<MessageReceiveActor>) -> bool {
        self.connections.write().unwrap().insert(addr)
    }

    pub fn remove(&mut self, addr: &Addr<MessageReceiveActor>) -> bool {
        self.connections.write().unwrap().remove(addr)
    }

    fn send_all(&self, msg: ResponseJson) {
        for addr in self.connections.read().unwrap().iter() {
            addr.do_send(msg.clone());
        }
    }
}

pub struct MessageReceiveActor {
    connections: ConnectionRepository,
}

impl MessageReceiveActor {
    pub fn new(connections: ConnectionRepository) -> Self {
        Self { connections }
    }
}

// TODO: Remove this after implementing Hub API
#[derive(Deserialize, Serialize, Debug, Clone, Message)]
#[rtype(result = "Result<(), ()>")]
struct ResponseJson {
    pub message_from: String,
    pub message_id: String,
    pub payload: serde_json::Value,
    pub received_at: u128,
}

// TODO: Remove this after implementing Hub API
async fn receive_message() -> Result<Vec<ResponseJson>, NodeXError> {
    let hub = Hub::new();
    let message = hub.get_message().await?;

    let mut response = Vec::new();
    for m in message.into_iter() {
        let json_message = serde_json::from_str(&m.raw_message).map_err(|e| {
            log::error!("Error: {:?}", e);
            NodeXError {}
        })?;
        match DIDCommEncryptedService::verify(&json_message).await {
            Ok(verified) => {
                let message = ResponseJson {
                    message_from: verified.message.issuer.id,
                    message_id: m.id,
                    payload: verified.message.credential_subject.container,
                    received_at: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis(),
                };
                response.push(message);
            }
            Err(e) => {
                log::error!("Error: {:?}", e);
                // TODO: add verify error response to hub.
                continue;
            }
        }
    }

    Ok(response)
}

impl Actor for MessageReceiveActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("Actor started");
        self.connections.insert(ctx.address());
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        log::info!("Actor is stopping");
        self.connections.remove(&ctx.address());
        Running::Stop
    }
}

impl Handler<ResponseJson> for MessageReceiveActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: ResponseJson, ctx: &mut Self::Context) -> Self::Result {
        let msg = serde_json::to_string(&msg).map_err(|e| log::error!("Error: {:?}", e))?;
        ctx.text(msg);

        Ok(())
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MessageReceiveActor {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match item {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Error: {:?}", e);
                ctx.stop();
                return;
            }
        };

        match msg {
            ws::Message::Ping(msg) => {
                log::info!("Ping: {:?}", msg);
                ctx.pong(&msg)
            }
            ws::Message::Text(text) => {
                log::info!("Received text: {}", text.to_string());
                ctx.text(text)
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Binary(_) | ws::Message::Continuation(_) => {
                log::error!("Not supported");
                ctx.stop();
            }
            ws::Message::Pong(_) | ws::Message::Nop => (),
        }
    }
}

// NOTE: GET /receive
#[derive(Deserialize, Serialize)]
struct MessageContainer {}

pub async fn handler(
    req: HttpRequest,
    context: web::Data<server::Context>,
    stream: web::Payload,
) -> actix_web::Result<HttpResponse> {
    let actor = MessageReceiveActor::new(context.connections.clone());
    let resp = ws::start(actor, &req, stream);
    log::info!("{:?}", resp);
    resp
}

pub async fn polling_task(
    shutdown_marker: Arc<AtomicBool>,
    connection_repository: ConnectionRepository,
) {
    log::info!("Polling task is started");
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    while !shutdown_marker.load(std::sync::atomic::Ordering::SeqCst) {
        interval.tick().await;
        match receive_message().await {
            Ok(messages) => messages
                .into_iter()
                .for_each(|msg| connection_repository.send_all(msg)),
            Err(e) => log::error!("Error: {:?}", e),
        }
    }

    log::info!("Polling task is stopped")
}
