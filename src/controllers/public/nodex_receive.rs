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

use crate::server;

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

    pub fn send_all(&self, msg: Message) {
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
pub struct Message {
    pub message_from: String,
    pub payload: String,
    pub received_at: u128,
}

// TODO: Remove this after implementing Hub API
pub async fn receive_message() -> Result<Message, ()> {
    log::info!("Receive message");
    Ok(Message {
        message_from: "did:example:123".to_string(),
        payload: "Hello World".to_string(),
        received_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    })
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

impl Handler<Message> for MessageReceiveActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        log::info!("Received message: {:?}", msg);
        let msg = serde_json::to_string(&msg).map_err(|e| {
            log::error!("Error: {:?}", e);
            ()
        })?;
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
                log::info!("Received text: {}", text);
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
        let message = receive_message().await;
        match message {
            Ok(msg) => connection_repository.send_all(msg),
            _ => unreachable!(),
        }
    }
    log::info!("Polling task is stopped")
}
