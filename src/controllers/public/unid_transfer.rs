use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, web };
use serde_json::Value;
use tokio::sync::oneshot;
use crate::{Context, Command};

// NOTE: POST /transfer
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    destinations: Vec<String>,
    messages: Vec<Value>,
    metadata: Value,
}

pub async fn handler(
    req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
    context: web::Data<Context>,
) -> actix_web::Result<HttpResponse> {
    let service = crate::services::unid::UNiD::new();

    // NOTE: We will provide an update soon to allow multiple destinations.
    if json.destinations.len() != 1 {
        return Ok(HttpResponse::InternalServerError().finish())
    }

    let to_did = match json.destinations.first() {
        Some(v) => v,
        _ => return Ok(HttpResponse::InternalServerError().finish())
    };

    match service.transfer(&to_did, &json.messages, &json.metadata).await {
        Ok(v) => {
            let (tx, rx) = oneshot::channel();

            let command = Command::Send {
                value: v.clone(),
                resp: tx,
            };

            if context.sender.lock().await.send(command).await.is_err() {
                return Ok(HttpResponse::InternalServerError().finish());
            }

            match rx.await {
                Ok(is_success) => {
                    if is_success {
                        Ok(HttpResponse::Ok().json(&v))
                    } else {
                        Ok(HttpResponse::InternalServerError().finish())
                    }
                },
                _ => {
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}