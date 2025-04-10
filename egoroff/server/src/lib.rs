#![warn(unused_extern_crates)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]

use anyhow::{Context, Result, anyhow};
use axum::Router;

use kernel::graph::{SiteGraph, SiteSection};
use std::env;
use std::sync::Arc;
use std::{fs::File, io::BufReader};
use std::{net::SocketAddr, path::PathBuf};
use tokio::signal;
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

static BASE_PATH: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(base_path);
static SITE_MAP: std::sync::LazyLock<Option<SiteSection>> = std::sync::LazyLock::new(make_site_map);

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

    if let Ok(listener) = tokio::net::TcpListener::bind(listen_socket).await {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
        {
            tracing::error!("Sever run failed with: {}", e);
        }
    } else {
        tracing::error!("Failed to start server at 0.0.0.0:{}", ports.http);
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        tracing::info!("ctrl_c signal received in task, starting graceful shutdown");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        tracing::info!("terminate signal received in task, starting graceful shutdown");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
