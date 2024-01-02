#![warn(unused_extern_crates)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]

use anyhow::{anyhow, Context, Result};
use axum::Router;

use axum::extract::Request;
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use kernel::graph::{SiteGraph, SiteSection};
use std::env;
use std::sync::Arc;
use std::{fs::File, io::BufReader};
use std::{net::SocketAddr, path::PathBuf};
use tokio::signal;
use tokio::sync::watch;
use tower::Service;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::domain::Config;
#[macro_use]
extern crate async_trait;

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
}

mod atom;
mod auth;
mod body;
mod domain;
mod handlers;
mod indie;
mod micropub;
mod rest;
mod sitemap;

pub const SESSIONS_DATABASE: &str = "egoroff_sessions.db";

lazy_static::lazy_static! {
    static ref BASE_PATH : PathBuf = base_path();
    static ref SITE_MAP : Option<SiteSection> = make_site_map();
}

fn base_path() -> PathBuf {
    if let Ok(d) = env::var("EGOROFF_HOME_DIR") {
        PathBuf::from(d)
    } else {
        std::env::current_dir().unwrap_or_default()
    }
}

fn make_site_map() -> Option<SiteSection> {
    let site_map_path = BASE_PATH.join("static/map.json");
    let file = match File::open(site_map_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("map.json open error: {e}");
            return None;
        }
    };
    let reader = BufReader::new(file);

    match serde_json::from_reader(reader) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("map.json cannot be converted into object: {e}");
            None
        }
    }
}

pub async fn run() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "server=debug,axum=debug,hyper=info,tower=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let http_port = env::var("EGOROFF_HTTP_PORT").unwrap_or_else(|_| String::from("4200"));
    let store_uri = env::var("EGOROFF_STORE_URI").unwrap_or_default();
    let certs_path = env::var("EGOROFF_CERT_DIR").unwrap_or_default();

    tracing::debug!("Base path {}", BASE_PATH.to_str().unwrap_or_default());

    let data_path = if let Ok(d) = env::var("EGOROFF_DATA_DIR") {
        PathBuf::from(d)
    } else {
        std::env::current_dir().unwrap_or_default()
    };
    tracing::debug!("Data path {}", data_path.to_str().unwrap_or_default());

    let config_path = BASE_PATH.join("static/config.json");
    let file = File::open(config_path).with_context(|| "config.json open error")?;
    let reader = BufReader::new(file);

    let site_config: Config = serde_json::from_reader(reader)
        .with_context(|| "config.json cannot be converted into object")?;

    let root = SITE_MAP
        .as_ref()
        .ok_or(anyhow!("Site root cannot be created"))?;
    let site_graph = Arc::new(SiteGraph::new(root));

    let app = rest::create_routes(
        BASE_PATH.to_path_buf(),
        site_graph,
        site_config,
        &data_path,
        store_uri,
        certs_path.clone(),
    )
    .with_context(|| "Routes creation error")?;

    let ports = Ports {
        http: http_port.parse().unwrap_or_default(),
    };

    let http = tokio::spawn(http_server(ports, app));

    // Ignore errors.
    let http_r = http.await;
    http_r.with_context(|| "Failed to start http server")?;
    Ok(())
}

async fn http_server(ports: Ports, app: Router) {
    let listen_socket = SocketAddr::from(([0, 0, 0, 0], ports.http));
    tracing::debug!("HTTP listening on {listen_socket}");

    let (close_tx, close_rx) = watch::channel(());

    if let Ok(listener) = tokio::net::TcpListener::bind(listen_socket).await {
        loop {
            let (client_stream, _remote_addr) = tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!("failed to accept connection: {e:#}");
                            continue;
                        },
                    }
                }
                () = shutdown_signal() => {
                    tracing::info!("signal received, not accepting new connections");
                    break;
                }
            };

            // We don't need to call `poll_ready` because `Router` is always ready.
            let tower_service = app.clone();

            let close_rx = close_rx.clone();

            tokio::spawn(async move {
                let client_socket = TokioIo::new(client_stream);

                // Hyper also has its own `Service` trait and doesn't use tower. We can use
                // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
                // `tower::Service::call`.
                let hyper_service =
                    hyper::service::service_fn(move |request: Request<Incoming>| {
                        // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
                        // tower's `Service` requires `&mut self`.
                        //
                        // We don't need to call `poll_ready` since `Router` is always ready.
                        tower_service.clone().call(request)
                    });

                // `hyper_util::server::conn::auto::Builder` supports both http1 and http2 but doesn't
                // support graceful so we have to use hyper directly and unfortunately pick between
                // http1 and http2.
                let conn = hyper::server::conn::http1::Builder::new()
                    .serve_connection(client_socket, hyper_service)
                    // `with_upgrades` is required for websockets.
                    .with_upgrades();

                let mut conn = std::pin::pin!(conn);

                loop {
                    tokio::select! {
                        result = conn.as_mut() => {
                            if let Err(err) = result {
                                tracing::error!("failed to serve connection: {err:#}");
                            }
                            break;
                        }
                        () = shutdown_signal() => {
                            tracing::info!("signal received in task, starting graceful shutdown");
                            conn.as_mut().graceful_shutdown();
                        }
                    }
                }
                drop(close_rx);
            });
        }
        drop(close_rx);
        drop(listener);
        close_tx.closed().await;
    } else {
        tracing::error!("Failed to start server at 0.0.0.0:{}", ports.http);
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
