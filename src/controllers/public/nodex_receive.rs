use actix::prelude::*;
use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

struct MessageReceiveActor {
    interval_handle: Option<SpawnHandle>,
}

impl MessageReceiveActor {
    pub fn new() -> Self {
        Self {
            interval_handle: None,
        }
    }
}

// TODO: Remove this after implementing Hub API
#[derive(Deserialize, Serialize, Debug)]
struct Message {
    pub message_from: String,
    pub payload: String,
    pub received_at: u128,
}

// TODO: Remove this after implementing Hub API
async fn receive_message() -> Result<Message, ()> {
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
        log::info!("Actor is alive");

        let handle = ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            receive_message()
                .into_actor(act)
                .then(|res, _, ctx| {
                    match res {
                        Ok(msg) => {
                            log::info!("Received message: {:?}", msg);
                            ctx.text(serde_json::to_string(&msg).unwrap());
                        }
                        _ => {
                            log::error!("Error");
                        }
                    }
                    fut::ready(())
                })
                .spawn(ctx);
        });

        self.interval_handle = Some(handle);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        log::info!("Actor is stopping");
        if let Some(handle) = self.interval_handle.take() {
            ctx.cancel_future(handle);
        }
        Running::Stop
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

pub async fn handler(req: HttpRequest, stream: web::Payload) -> actix_web::Result<HttpResponse> {
    let actor = MessageReceiveActor::new();
    let resp = ws::start(actor, &req, stream);
    log::info!("{:?}", resp);
    resp
}
