use axum::extract::DefaultBodyLimit;
use axum::Extension;
use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use domain::PageContext;
use kernel::graph::{SiteGraph, SiteSection};
use kernel::typograph;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use std::{fs::File, io::BufReader};
use std::{net::SocketAddr, path::PathBuf};
use tera::{try_get_value, Tera};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::domain::Config;

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

mod domain;
mod handlers;

pub async fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "server=debug,axum=debug,hyper=debug,tower=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let http_port = env::var("EGOROFF_HTTP_PORT").unwrap_or_else(|_| String::from("4200"));
    let https_port = env::var("EGOROFF_HTTPS_PORT").unwrap_or_else(|_| String::from("4201"));

    let base_path = if let Ok(d) = env::var("EGOROFF_HOME_DIR") {
        PathBuf::from(d)
    } else {
        std::env::current_dir().unwrap()
    };
    tracing::debug!("Base path {}", base_path.to_str().unwrap());

    let data_path = if let Ok(d) = env::var("EGOROFF_DATA_DIR") {
        PathBuf::from(d)
    } else {
        std::env::current_dir().unwrap()
    };
    tracing::debug!("Data path {}", data_path.to_str().unwrap());

    let config_path = base_path.join("static/config.json");
    let file = File::open(config_path).unwrap();
    let reader = BufReader::new(file);
    let site_config: Config = serde_json::from_reader(reader).unwrap();

    let site_map_path = base_path.join("static/map.json");
    let file = File::open(site_map_path).unwrap();
    let reader = BufReader::new(file);

    let root: SiteSection = serde_json::from_reader(reader).unwrap();
    let site_graph = SiteGraph::new(root);
    let site_graph_clone = site_graph.clone();

    let templates_path = base_path.join("static/dist/**/*.html");
    let templates_path = templates_path.to_str().unwrap();

    let mut tera = match Tera::new(templates_path) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Server error: {e}");
            return;
        }
    };

    tera.register_function(
        "path_for",
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("id") {
                Some(val) => match tera::from_value::<String>(val.clone()) {
                    Ok(v) => Ok(tera::to_value(site_graph.full_path(&v)).unwrap()),
                    Err(_) => Err("oops".into()),
                },
                None => Err("oops".into()),
            }
        },
    );

    tera.register_filter("typograph", typograph);

    let app = create_routes(base_path, site_graph_clone, site_config, tera, data_path);

    let ports = Ports {
        http: http_port.parse().unwrap(),
        https: https_port.parse().unwrap(),
    };

    let handle = Handle::new();

    let https = tokio::spawn(https_server(ports, handle.clone(), app.clone()));
    let http = tokio::spawn(http_server(ports, handle, app));

    // Ignore errors.
    let _ = tokio::join!(http, https);
}

async fn http_server(ports: Ports, handle: Handle, app: Router) {
    let addr = SocketAddr::from(([0, 0, 0, 0], ports.http));
    tracing::debug!("HTTP listening on {}", addr);

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(shutdown_signal(handle.clone()));

    axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn https_server(ports: Ports, handle: Handle, app: Router) {
    let addr: SocketAddr = format!("0.0.0.0:{}", ports.https).parse().unwrap();
    tracing::debug!("HTTPS listening on {addr}");

    // configure certificate and private key used by https
    let cert_dir = env::var("EGOROFF_CERT_DIR").unwrap_or_else(|_| String::from("."));
    tracing::debug!("Certs path {cert_dir}");
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(&cert_dir).join("egoroff_spb_ru.crt"),
        PathBuf::from(&cert_dir).join("egoroff_spb_ru.key.pem"),
    )
    .await
    .expect("Certificate cannot be loaded");

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(shutdown_signal(handle.clone()));

    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub fn create_routes(
    base_path: PathBuf,
    site_graph: SiteGraph,
    site_config: Config,
    tera: Tera,
    data_path: PathBuf,
) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let storage_path = data_path.join(kernel::sqlite::DATABASE);

    let page_context = PageContext {
        base_path,
        storage_path,
        tera,
        site_graph,
        site_config,
    };

    Router::new()
        .route("/", get(handlers::serve_index))
        .route("/portfolio/", get(handlers::serve_portfolio))
        .route("/portfolio/:path", get(handlers::serve_portfolio_document))
        .route(
            "/portfolio/apache/:path",
            get(handlers::serve_portfolio_document),
        )
        .route(
            "/portfolio/portfolio/:path",
            get(handlers::serve_portfolio_document),
        )
        .route("/blog/", get(handlers::serve_blog_default))
        .route(
            "/blog/page/:page",
            get(handlers::serve_blog_not_default_page),
        )
        .route(
            "/blog/page/:page/",
            get(handlers::serve_blog_not_default_page),
        )
        .route("/blog/:path", get(handlers::serve_blog_page))
        .route("/search/", get(handlers::serve_search))
        .route("/:path", get(handlers::serve_root))
        .route("/js/:path", get(handlers::serve_js))
        .route("/css/:path", get(handlers::serve_css))
        .route("/img/:path", get(handlers::serve_img))
        .route("/apache/:path", get(handlers::serve_apache))
        .route("/apache/images/:path", get(handlers::serve_apache_images))
        .route("/api/v2/navigation/", get(handlers::navigation))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http().on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        tracing::error!("Server error: {error}");
                    },
                ))
                .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
                .into_inner(),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(20 * 1024 * 1024))
        .layer(Extension(page_context))
        .layer(prometheus_layer)
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
    let result = typograph::typograph(s);
    match result {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Err(tera::Error::from(e.to_string())),
    }
}
