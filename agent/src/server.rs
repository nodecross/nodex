use crate::controllers;
use axum::{
    routing::{get, post},
    Router,
};

#[cfg(unix)]
pub mod unix {
    use axum::http::Request;
    use axum::Router;
    use hyper::body::Incoming;
    use hyper_util::{
        rt::{TokioExecutor, TokioIo},
        server,
    };
    use std::convert::Infallible;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
    use std::path::Path;
    use tokio::net::{UnixListener, UnixStream};
    use tokio::signal::unix::{signal, SignalKind};
    use tokio::task::JoinSet;
    use tokio_util::sync::CancellationToken;
    use tower::Service;

    fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => match err {},
        }
    }

    pub fn make_listener(uds_path: impl AsRef<Path>) -> std::io::Result<UnixListener> {
        let meta_uds_path = controller::convention_of_meta_uds_path(&uds_path)?;
        remove_file_if_exists(&meta_uds_path);
        let sock = std::os::unix::net::UnixListener::bind(&meta_uds_path)?;
        let permissions = std::fs::Permissions::from_mode(0o766);
        std::fs::set_permissions(&meta_uds_path, permissions)?;
        let (stream, _) = sock.accept()?;
        let fd = controller::managers::agent::recv_fd(stream.as_raw_fd())?;
        remove_file_if_exists(&meta_uds_path);
        let uds = match fd {
            Some(fd) => {
                let listener =
                    unsafe { std::os::unix::net::UnixListener::from_raw_fd(fd as RawFd) };
                UnixListener::from_std(listener)?
            }
            None => {
                remove_file_if_exists(&uds_path);
                UnixListener::bind(&uds_path)?
            }
        };
        Ok(uds)
    }

    pub async fn make_uds_server(router: Router, uds: UnixListener) -> std::io::Result<()> {
        // https://github.com/tokio-rs/axum/blob/main/examples/unix-domain-socket/src/main.rs
        let mut app = router.into_make_service();
        loop {
            let (socket, _remote_addr) = uds.accept().await?;
            let tower_service = unwrap_infallible(app.call(&socket).await);
            tokio::spawn(async move {
                let socket = TokioIo::new(socket);
                let hyper_service =
                    hyper::service::service_fn(move |request: Request<Incoming>| {
                        tower_service.clone().call(request)
                    });
                if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection_with_upgrades(socket, hyper_service)
                    .await
                {
                    eprintln!("failed to serve connection: {err:#}");
                }
            });
        }
    }

    fn remove_file_if_exists(path: impl AsRef<Path>) {
        if path.as_ref().exists() {
            let _ = std::fs::remove_file(path);
        }
    }

    pub fn wrap_with_signal_handler(
        server: impl std::future::Future<Output = std::io::Result<()>> + Send + 'static,
        token: CancellationToken,
        fd: RawFd,
        uds_path: impl AsRef<Path>,
    ) -> JoinSet<std::io::Result<()>> {
        let mut set = JoinSet::new();
        let cloned_token = token.clone();
        let tasks = async move {
            tokio::select! {
                _ = cloned_token.cancelled() => Ok(()),
                res = server => res,
            }
        };
        set.spawn(tasks);
        let uds_path = uds_path.as_ref().to_owned();
        set.spawn(async move {
            let ctrl_c = tokio::signal::ctrl_c();
            let mut sigterm = signal(SignalKind::terminate())?;
            let mut sigusr1 = signal(SignalKind::user_defined1())?;
            tokio::select! {
                _ = ctrl_c => {
                    log::info!("Received Ctrl+C");
                    token.cancel();
                    Ok(())
                },
                _ = sigterm.recv() => {
                    log::info!("Received SIGTERM");
                    token.cancel();
                    Ok(())
                },
                _ = sigusr1.recv() => {
                    log::info!("Received SIGUSR1");
                    let send_sock_path = controller::convention_of_meta_uds_path(&uds_path)?;
                    let () = controller::managers::agent::wait_until_file_created(&send_sock_path)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, format!("{}", e)))?;
                    let stream = UnixStream::connect(&send_sock_path).await?;
                    controller::managers::agent::send_fd(stream.as_raw_fd(), Some(fd))?;
                    token.cancel();
                    Ok(())
                }
            }
        });
        set
    }
}

#[cfg(windows)]
pub fn new_web_server<C: TransferClient + 'static>(port: u16, sender: C) -> Server {
    let context = web::Data::new(Context {
        sender: TokioMutex::new(sender),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::DefaultHeaders::new().add(("x-version", "0.1.0")))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .configure(config_app(&context))
    })
    .bind(format!("127.0.0.1:{}", port))
    .unwrap()
    .workers(1)
    .run()
}

pub fn make_router() -> Router {
    Router::new()
        .route(
            "/identifiers",
            post(controllers::public::nodex_create_identifier::handler),
        )
        .route(
            "/identifiers/:did",
            get(controllers::public::nodex_find_identifier::handler),
        )
        .route(
            "/create-verifiable-message",
            post(controllers::public::nodex_create_verifiable_message::handler),
        )
        .route(
            "/verify-verifiable-message",
            post(controllers::public::nodex_verify_verifiable_message::handler),
        )
        .route(
            "/create-didcomm-message",
            post(controllers::public::nodex_create_didcomm_message::handler),
        )
        .route(
            "/verify-didcomm-message",
            post(controllers::public::nodex_verify_didcomm_message::handler),
        )
        .route("/events", post(controllers::public::send_event::handler))
        .route(
            "/custom-metrics",
            post(controllers::public::send_custom_metric::handler),
        )
        .route(
            "/attributes",
            post(controllers::public::send_attribute::handler),
        )
        // NOTE: Internal (Private) Routes
        .route(
            "/internal/version/get",
            get(controllers::internal::version::handler_get),
        )
        .route(
            "/internal/version/update",
            post(controllers::internal::version::handler_update),
        )
        .route(
            "/internal/network",
            post(controllers::internal::network::handler),
        )
}
