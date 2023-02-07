#![feature(libc)]
#![feature(once_cell)]
#![feature(const_option)]
#![feature(default_alloc_error_handler)]
#![feature(vec_into_raw_parts)]
#![feature(trivial_bounds)]

extern crate env_logger;

use actix_web::{ middleware, HttpServer, App, web, dev::Server };
use clap::Parser;
use daemonize::Daemonize;
use rumqttc::{AsyncClient, QoS, EventLoop, Event, Packet, MqttOptions};
use serde::{Deserialize, Serialize};
use services::unid::UNiD;
use tokio::sync::{mpsc, Mutex as TokioMutex};
use tokio::sync::{mpsc::{Sender, Receiver}, RwLock, oneshot};
use tokio::time::{Instant, Duration, sleep};
use unid::{extension::secure_keystore::{SecureKeyStore, SecureKeyStoreType}};
use serde_json::{json, Value};
use std::sync::atomic::AtomicBool;
use std::{fs::{File, self}, path::PathBuf, sync::{Arc, Once, Mutex}, collections::HashMap};
use dirs;
use cuid;

use crate::config::AppConfig;

mod unid;
mod services;
mod config;
mod controllers;

#[derive(Clone)]
pub struct SingletonAppConfig {
    inner: Arc<Mutex<AppConfig>>,
}

pub fn app_config() -> Box<SingletonAppConfig> {
    static mut SINGLETON: Option<Box<SingletonAppConfig>> = None;
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let singleton = SingletonAppConfig {
                inner: Arc::new(Mutex::new(AppConfig::new()))
            };

            SINGLETON = Some(Box::new(singleton))
        });

        SINGLETON.clone().unwrap()
    }
}

#[derive(Parser, Debug)]
#[clap(name = "unid-agent")]
#[clap(name = "unid-agent")]
#[clap(version, about, long_about = None)]
struct Args {
    /// Run as daemon mode
    #[clap(short, long)]
    daemonize: bool,
}

type Responder = oneshot::Sender<bool>;

#[derive(Debug)]
enum Command {
    Send {
        value: Value,
        resp: Responder,
    }
}

#[derive(Debug)]
pub struct Context {
    sender: TokioMutex<Sender<Command>>
}

async fn sender_handler(mut rx: Receiver<Command>, client: AsyncClient, db: Arc<RwLock<HashMap::<String, bool>>>, topic: String) {
    log::info!("start sender");

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Send { value, resp } => {
                let id = cuid::cuid2();

                let payload: Value = json!({
                    "id": id,
                    "value": value,
                });

                match client.publish(topic.to_string(), QoS::AtLeastOnce, false, payload.to_string().as_bytes()).await {
                    Ok(_) => {
                        db.write().await.insert(id.clone(), false);

                        let start = Instant::now();
                        let threshold = Duration::from_secs(15);

                        loop {
                            if threshold < start.elapsed() {
                                resp.send(false);
                                break
                            }

                            match db.read().await.get(&id) {
                                Some(v) => {
                                    if *v {
                                        resp.send(true);
                                        break
                                    }
                                },
                                None => {
                                    continue;
                                }
                            }

                            sleep(Duration::from_secs(1)).await;
                        }
                    },
                    _ => { }
                }
            }
        }
    }

    log::info!("stop sender");
}

async fn receiver_handler(shutdown_marker: Arc<AtomicBool>, mut eventloop: EventLoop, db: Arc<RwLock<HashMap::<String, bool>>>) {
    #[derive(Debug, Serialize, Deserialize)]
    struct Response {
        received_id: String,
    }

    log::info!("start receiver");

    while let Ok(notification) = eventloop.poll().await {
        if shutdown_marker.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        match notification {
            Event::Incoming(v) => {
                match v {
                    Packet::Publish(v) => {
                        match serde_json::from_slice::<Response>(&v.payload) {
                            Ok(payload) => {
                                let mut keys = Vec::<String>::new();

                                db.read().await.keys().enumerate().for_each(|v| {
                                    keys.push(v.1.to_string());
                                });

                                let item = keys.iter().find(|v| v.to_string() == payload.received_id);

                                if let Some(v) = item {
                                    println!("matched: {}", v);
                                    let _ = db.write().await.insert(v.to_string(), true);
                                }
                            },
                            _ => {},
                        };
                    },
                    _ => {},
                }
            },
            Event::Outgoing(_) => {}
        }
    }

    log::info!("stop receiver");
}

fn new_server(sock_path: &PathBuf, sender: Sender<Command>) -> Server {
    let context = web::Data::new(Context {
        sender: TokioMutex::new(sender),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::DefaultHeaders::new().add(("x-version", "0.1.0")))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .app_data(context.clone())

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
    .bind_uds(&sock_path)
    .unwrap()
    .workers(1)
    .run()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let hub_did_topic = "nodex/did:unid:test:EiCW6eklabBIrkTMHFpBln7574xmZlbMakWSCNtBWcunDg";

    let config = AppConfig::new();
    match config.write() {
        Ok(()) => (),
        Err(_) => panic!(),
    };

    let home_dir = match dirs::home_dir() {
        Some(v) => v,
        None => panic!(),
    };
    let config_dir = home_dir.join(".unid");
    let runtime_dir = config_dir.clone().join("run");
    let logs_dir = config_dir.clone().join("logs");

    match fs::create_dir_all(&runtime_dir) {
        Ok(()) => (),
        Err(_) => panic!(),
    };
    match fs::create_dir_all(&logs_dir) {
        Ok(()) => (),
        Err(_) => panic!(),
    };

    let stdout = File::create(logs_dir.clone().join("unid.log")).unwrap();
    let stderr = File::create(logs_dir.clone().join("unid.err")).unwrap();
    let sock_path = runtime_dir.clone().join("unid.sock");

    let daemonize = Daemonize::new()
        .pid_file(runtime_dir.clone().join("unid.pid"))
        .working_directory(".")
        .stdout(stdout)
        .stderr(stderr);

    let args = Args::parse();

    // NOTE: generate Key Chain
    let node_x = UNiD::new();
    let did = node_x.create_identifier().await.unwrap();

    // NOTE: connect mqtt server
    let mqtt_host = "demo-mqtt.getunid.io";
    let mqtt_port = 1883;
    let mqtt_client_id = cuid::cuid2();

    let did_id = did.did_document.id;
    let mqtt_topic = format!("nodex/{}", did_id);

    let mut mqtt_options = MqttOptions::new(&mqtt_client_id, mqtt_host, mqtt_port);
    mqtt_options.set_clean_session(true);
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

    client.subscribe(hub_did_topic, QoS::ExactlyOnce).await.unwrap();
    log::info!("subscribed: {}", hub_did_topic);

    // NOTE: booting...
    let (tx, rx) = mpsc::channel::<Command>(32);
    let db = Arc::new(RwLock::new(HashMap::<String, bool>::new()));

    let server = new_server(&sock_path, tx);
    let server_handle = server.handle();

    let shutdown_marker = Arc::new(AtomicBool::new(false));

    let server_task = tokio::spawn(server);
    let sender_task = tokio::spawn(sender_handler(rx, client, Arc::clone(&db), mqtt_topic));
    let receiver_task = tokio::spawn(receiver_handler(Arc::clone(&shutdown_marker), eventloop, Arc::clone(&db)));

    let shutdown = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();

        let server_stop = server_handle.stop(true);
        shutdown_marker.store(true, std::sync::atomic::Ordering::SeqCst);

        server_stop.await;
    });

    if args.daemonize {
        match daemonize.start() {
            Ok(_) => {
                let _ = tokio::try_join!(server_task, sender_task, receiver_task, shutdown);
            },
            Err(_) => panic!(),
        }
    } else {
        let _ = tokio::try_join!(server_task, sender_task, receiver_task, shutdown);
    }

    Ok(())
}
