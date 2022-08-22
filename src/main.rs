#![feature(libc)]
#![feature(once_cell)]
#![feature(const_option)]
#![feature(default_alloc_error_handler)]
#![feature(vec_into_raw_parts)]
#![feature(trivial_bounds)]

extern crate env_logger;

use actix_web::{ middleware, HttpServer, App, web };
use clap::Parser;
use daemonize::Daemonize;
use std::{fs::{File, self}, io};

mod unid;
mod services;
mod config;
mod controllers;

#[derive(Parser, Debug)]
#[clap(name = "unid-agent")]
#[clap(name = "unid-agent")]
#[clap(version, about, long_about = None)]
struct Args {
    /// Run as daemon mode
    #[clap(short, long)]
    daemonize: bool,
}

#[actix_web::main]
async fn run() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().add(("x-version", "0.1.0")))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())

            // NOTE: Public Routes
            .route("/identifiers", web::post().to(controllers::public::unid_create_identifier::handler))
            .route("/identifiers/{did}", web::get().to(controllers::public::unid_find_identifier::handler))
            .route("/transfer", web::post().to(controllers::public::unid_transfer::handler))

            // NOTE: Internal (Private) Routes
            .route("/internal/verifiable-credentials", web::post().to(controllers::internal::did_generate_vc::handler))
            .route("/internal/verifiable-credentials/verify", web::post().to(controllers::internal::did_verify_vc::handler))
            .route("/internal/verifiable-presentations", web::post().to(controllers::internal::did_generate_vp::handler))
            .route("/internal/verifiable-presentations/verify", web::post().to(controllers::internal::did_verify_vp::handler))

            .route("/internal/didcomm/plaintext-messages", web::post().to(controllers::internal::didcomm_generate_plaintext::handler))
            .route("/internal/didcomm/plaintext-messages/verify", web::post().to(controllers::internal::didcomm_verify_plaintext::handler))
            .route("/internal/didcomm/signed-messages", web::post().to(controllers::internal::didcomm_generate_signed::handler))
            .route("/internal/didcomm/signed-messages/verify", web::post().to(controllers::internal::didcomm_verify_signed::handler))
            .route("/internal/didcomm/encrypted-messages", web::post().to(controllers::internal::didcomm_generate_encrypted::handler))
            .route("/internal/didcomm/encrypted-messages/verify", web::post().to(controllers::internal::didcomm_verify_encrypted::handler))
    })
    .bind_uds("unid-agent.sock")?
    .workers(1)
    .run()
    .await
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let stdout = File::create("unid-agent.log").unwrap();
    let stderr = File::create("unid-agent.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("unid-agent.pid")
        .working_directory(".")
        .stdout(stdout)
        .stderr(stderr);

    let args = Args::parse();

    if args.daemonize {
        match daemonize.start() {
            Ok(_) => {
                run()
            },
            Err(_) => {
                Err(io::Error::new(io::ErrorKind::Other, ""))
            }
        }
    } else {
        run()
    }
}