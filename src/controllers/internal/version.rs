use crate::nodex::errors::NodeXError;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, process::Command};

// NOTE: POST /internal/version
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: Value,
}

pub async fn handler_get(_req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let current_version = env!("CARGO_PKG_VERSION");
    Ok(HttpResponse::Ok().json(&json!({ "version": current_version })))
}

pub async fn handler_update(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let binary_url = match json.message["binary_url"].as_str() {
        Some(url) => url,
        None => return Ok(HttpResponse::BadRequest().json("binary_url is required")),
    };
    let path = match json.message["path"].as_str() {
        Some(p) => p,
        None => return Ok(HttpResponse::BadRequest().json("path is required")),
    };
    match check_for_updates(binary_url, path).await {
        Ok(_) => Ok(HttpResponse::Ok().json("ok")),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

async fn check_for_updates(binary_url: &str, path: &str) -> Result<(), NodeXError> {
    let response = reqwest::get(binary_url).await;
    match response {
        Ok(r) => {
            let content = match r.bytes().await {
                Ok(c) => c,
                Err(_) => return Err(NodeXError {}),
            };
            match fs::write(path, &content) {
                Ok(_) => (),
                Err(_) => return Err(NodeXError {}),
            };
            match Command::new("chmod").arg("+x").arg(path).status() {
                Ok(_) => (),
                Err(_) => return Err(NodeXError {}),
            };
            match Command::new(path).spawn() {
                Ok(_) => (),
                Err(_) => return Err(NodeXError {}),
            };
            Ok(())
        }
        Err(_) => Err(NodeXError {}),
    }
}
