use axum::{
    extract::Host,
    handler::Handler,
    http::{StatusCode, Uri},
    response::Redirect,
    routing::get,
    BoxError, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use std::env;
use std::time::Duration;
use std::{net::SocketAddr, path::PathBuf};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::Span;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

pub async fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let http_port = env::var("EGOROFF_HTTP_PORT").unwrap_or_else(|_| String::from("4200"));
    let https_port = env::var("EGOROFF_HTTPS_PORT").unwrap_or_else(|_| String::from("4201"));

    let ports = Ports {
        http: http_port.parse().unwrap(),
        https: https_port.parse().unwrap(),
    };

    let handle = Handle::new();

    let https = tokio::spawn(https_server(ports, handle.clone()));
    let http = tokio::spawn(http_server(ports, handle));

    // Ignore errors.
    let _ = tokio::join!(http, https);
}

async fn http_server(ports: Ports, handle: Handle) {
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], ports.http));
    tracing::debug!("http redirect listening on {}", addr);

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(shutdown_signal(handle.clone()));

    axum_server::bind(addr)
        .handle(handle)
        .serve(redirect.into_make_service())
        .await
        .unwrap();
}

async fn https_server(ports: Ports, handle: Handle) {
    let addr: SocketAddr = format!("0.0.0.0:{}", ports.https).parse().unwrap();
    tracing::debug!("listening on {addr}");

    // configure certificate and private key used by https
    let cert_dir = env::var("EGOROFF_CERT_DIR").unwrap_or_else(|_| String::from("."));
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(&cert_dir).join("egoroff_spb_ru.crt"),
        PathBuf::from(&cert_dir).join("egoroff_spb_ru.key.pem"),
    )
    .await
    .expect("Certificate cannot be loaded");

    let app = create_routes();

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(shutdown_signal(handle.clone()));

    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(handlers::serve_index))
        .route("/js/:path", get(handlers::serve_js))
        .route("/css/:path", get(handlers::serve_css))
        .route("/img/:path", get(handlers::serve_img))
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

mod handlers {
    use axum::{
        body::{Empty, Full},
        extract,
        http::{HeaderValue, StatusCode},
        response::{Html, IntoResponse},
    };
    use rust_embed::RustEmbed;
    use tera::{Context, Tera};

    #[derive(RustEmbed)]
    #[folder = "../../static/dist/css"]
    struct Css;

    #[derive(RustEmbed)]
    #[folder = "../../static/dist/js"]
    struct Js;

    #[derive(RustEmbed)]
    #[folder = "../../static/img"]
    struct Img;

    pub async fn serve_index() -> impl IntoResponse {
        let tera = match Tera::new("/home/egr/code/egoroff.spb.ru/static/dist/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Server error: {e}");
                return Html(format!("Parsing error(s): {:#?}", e));
            }
        };
        let mut context = Context::new();
        context.insert("html_class", "welcome");
        context.insert("title", "egoroff.spb.ru");
        let messages: Vec<String> = Vec::new();
        context.insert("flashed_messages", &messages);
        context.insert("gin_mode", "debug");
        context.insert("ctx", "");
        let index = tera.render("welcome.html", &context);
        match index {
            Ok(content) => Html(content),
            Err(err) => {
                tracing::error!("Server error: {err}");
                Html(format!("{:#?}", err))
            }
        }
    }

    pub async fn serve_js(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
        let path = path.as_str();
        let asset = Js::get(path);
        get_embed(path, asset)
    }

    pub async fn serve_css(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
        let path = path.as_str();
        let asset = Css::get(path);
        get_embed(path, asset)
    }

    pub async fn serve_img(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
        let path = path.as_str();
        let asset = Img::get(path);
        get_embed(path, asset)
    }

    fn get_embed(path: &str, asset: Option<rust_embed::EmbeddedFile>) -> impl IntoResponse {
        if let Some(file) = asset {
            let mut res = Full::from(file.data).into_response();
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            res.headers_mut().insert(
                "content-type",
                HeaderValue::from_str(mime.as_ref()).unwrap(),
            );
            (StatusCode::OK, res)
        } else {
            (StatusCode::NOT_FOUND, Empty::new().into_response())
        }
    }
}
