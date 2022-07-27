use axum::{
    extract::Host,
    response::Redirect,
    http::{StatusCode, Uri},
    BoxError, Router,
    handler::Handler, routing::get,
};
use axum_server::tls_rustls::RustlsConfig;
use std::env;
use std::{net::SocketAddr, path::PathBuf};
use std::time::Duration;
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

    tokio::spawn(redirect_http_to_https(ports));

    let socket: SocketAddr = format!("0.0.0.0:{https_port}").parse().unwrap();
    tracing::debug!("listening on {socket}");

    // configure certificate and private key used by https
    let cert_dir = env::var("EGOROFF_CERT_DIR").unwrap_or_else(|_| String::from("."));
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(&cert_dir)
            .join("egoroff_spb_ru.crt"),
        PathBuf::from(&cert_dir)
            .join("egoroff_spb_ru.key.pem"),
    )
    .await
    .expect("Certificate cannot be loaded");

    let app = create_routes();

    axum_server::bind_rustls(socket, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn redirect_http_to_https(ports: Ports) {
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

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    tracing::debug!("http redirect listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(redirect.into_make_service())
        .await
        .unwrap();
}

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(handlers::serve_index))
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

mod handlers {
    use tera::{Tera, Context};
    use axum::response::{IntoResponse, Html};

    pub async fn serve_index() -> impl IntoResponse {
        let tera = match Tera::new("/home/egr/code/egoroff.spb.ru/static/dist/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                return Html(format!("Parsing error(s): {:#?}", e));
            }
        };
        let mut context = Context::new();
        context.insert("html_class", "welcome");
        context.insert("title", "egoroff.spb.ru");
        let messages : Vec<String> = Vec::new();
        context.insert("flashed_messages", &messages);
        context.insert("gin_mode", "debug");
        context.insert("ctx", "");
        let index = tera.render( "welcome.html", &context);
        match index {
            Ok(content) => Html(content),
            Err(err) => Html(format!("{:#?}", err)),
        }
    }
}