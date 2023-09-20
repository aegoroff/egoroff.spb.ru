#![warn(unused_extern_crates)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]

use anyhow::{anyhow, Context, Result};
use axum::Router;

use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use axum_sessions::{SameSite, SessionLayer};
use domain::{PageContext, RequireAuth};
use futures::lock::Mutex;
use indie::RequireIndieAuthorizationLayer;
use kernel::domain::SmallPost;
use kernel::graph::{SiteGraph, SiteSection};
use kernel::session::SqliteSessionStore;
use kernel::sqlite::{Mode, Sqlite};
use kernel::typograph;
use rand::Rng;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{fs::File, io::BufReader};
use std::{net::SocketAddr, path::PathBuf};
use tera::{try_get_value, Tera};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::compression::predicate::NotForContentType;
use tower_http::compression::{CompressionLayer, DefaultPredicate, Predicate};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::domain::Config;
#[macro_use]
extern crate async_trait;

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
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
    let https_port = env::var("EGOROFF_HTTPS_PORT").unwrap_or_else(|_| String::from("4201"));
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

    let templates_path = BASE_PATH.join("static/dist/**/*[a-zA-Z0-9][a-zA-Z0-9_].html");
    let templates_path = templates_path.to_str().unwrap_or_default();

    let mut tera = Tera::new(templates_path).with_context(|| "Tera templates cannot be created")?;

    tera.register_filter("typograph", typograph);
    tera.register_filter("human_readable_size", human_readable_size);

    let tera = Arc::new(tera);

    let app = rest::create_routes(
        BASE_PATH.to_path_buf(),
        site_graph,
        site_config,
        tera,
        &data_path,
        store_uri,
        certs_path.clone(),
    )
    .with_context(|| "Routes creation error")?;

    let ports = Ports {
        http: http_port.parse().unwrap_or_default(),
        https: https_port.parse().unwrap_or_default(),
    };

    let handle = Handle::new();

    let https = tokio::spawn(https_server(ports, handle.clone(), app.clone(), certs_path));
    let http = tokio::spawn(http_server(ports, handle, app));

    // Ignore errors.
    let (http_r, https_r) = tokio::join!(http, https);
    http_r.with_context(|| "Failed to start http server")?;
    https_r.with_context(|| "Failed to start https server")?;
    Ok(())
}

async fn http_server(ports: Ports, handle: Handle, app: Router) {
    let addr = SocketAddr::from(([0, 0, 0, 0], ports.http));
    tracing::debug!("HTTP listening on {}", addr);

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(shutdown_signal(handle.clone()));

    match axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service())
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                "Failed to start server at 0.0.0.0:{}. Error: {e}",
                ports.http
            );
        }
    }
}

async fn https_server(ports: Ports, handle: Handle, app: Router, certs_path: String) {
    let addr = SocketAddr::from(([0, 0, 0, 0], ports.https));
    tracing::debug!("HTTPS listening on {addr}");

    tracing::debug!("Certs path {certs_path}");
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(&certs_path).join("egoroff_spb_ru.crt"),
        PathBuf::from(&certs_path).join("egoroff_spb_ru.key.pem"),
    )
    .await
    .expect("Certificate cannot be loaded");

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(shutdown_signal(handle.clone()));

    match axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                "Failed to start server at 0.0.0.0:{}. Error: {e}",
                ports.https
            );
        }
    }
}

async fn shutdown_signal(handle: Handle) {
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(Duration::from_secs(2)));
}

fn typograph(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("typograph", "value", String, value);
    match typograph::typograph(&s) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Err(tera::Error::from(e.to_string())),
    }
}

fn human_readable_size(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let bytes = try_get_value!("human_readable_size", "value", u64, value);
    let s = human_bytes::human_bytes(bytes as f64);
    Ok(Value::String(s))
}
