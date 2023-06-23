#![warn(unused_extern_crates)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]

use auth::{GithubAuthorizer, GoogleAuthorizer, Role, UserStorage, YandexAuthorizer};
use axum::extract::DefaultBodyLimit;
use axum::handler::Handler;
use axum::routing::{delete, post, put};
use axum::Extension;
use axum::{routing::get, Router};

use axum_login::AuthLayer;

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
mod sitemap;

pub const SESSIONS_DATABASE: &str = "egoroff_sessions.db";

pub async fn run() {
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

    let base_path = if let Ok(d) = env::var("EGOROFF_HOME_DIR") {
        PathBuf::from(d)
    } else {
        std::env::current_dir().unwrap_or_default()
    };
    tracing::debug!("Base path {}", base_path.to_str().unwrap_or_default());

    let data_path = if let Ok(d) = env::var("EGOROFF_DATA_DIR") {
        PathBuf::from(d)
    } else {
        std::env::current_dir().unwrap_or_default()
    };
    tracing::debug!("Data path {}", data_path.to_str().unwrap_or_default());

    let config_path = base_path.join("static/config.json");
    let file = match File::open(config_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("config.json open error: {e}");
            return;
        }
    };
    let reader = BufReader::new(file);
    let site_config: Config = match serde_json::from_reader(reader) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("config.json cannot be converted into object: {e}");
            return;
        }
    };

    let site_map_path = base_path.join("static/map.json");
    let file = match File::open(site_map_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("map.json open error: {e}");
            return;
        }
    };
    let reader = BufReader::new(file);

    let root: SiteSection = match serde_json::from_reader(reader) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("map.json cannot be converted into object: {e}");
            return;
        }
    };
    let site_graph = Arc::new(SiteGraph::new(root));
    let site_graph_clone = site_graph.clone();

    let templates_path = base_path.join("static/dist/**/*[a-zA-Z0-9][a-zA-Z0-9_].html");
    let templates_path = templates_path.to_str().unwrap_or_default();

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
                    Ok(v) => match tera::to_value(site_graph.full_path(&v)) {
                        Ok(v) => Ok(v),
                        Err(_) => Err("oops".into()),
                    },
                    Err(_) => Err("oops".into()),
                },
                None => Err("oops".into()),
            }
        },
    );

    tera.register_filter("typograph", typograph);

    let tera = Arc::new(tera);

    let app = create_routes(
        base_path,
        site_graph_clone,
        site_config,
        tera,
        &data_path,
        store_uri,
        certs_path.clone(),
    );

    let ports = Ports {
        http: http_port.parse().unwrap_or_default(),
        https: https_port.parse().unwrap_or_default(),
    };

    let handle = Handle::new();

    let https = tokio::spawn(https_server(ports, handle.clone(), app.clone(), certs_path));
    let http = tokio::spawn(http_server(ports, handle, app));

    // Ignore errors.
    let _ = tokio::join!(http, https);
}

async fn http_server(ports: Ports, handle: Handle, app: Router) {
    let addr = SocketAddr::from(([0, 0, 0, 0], ports.http));
    tracing::debug!("HTTP listening on {}", addr);

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(shutdown_signal(handle.clone()));

    if let Ok(r) = axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service())
        .await
    {
        r
    } else {
        tracing::error!("Failed to start server at 0.0.0.0:{}", ports.http);
    }
}

async fn https_server(ports: Ports, handle: Handle, app: Router, certs_path: String) {
    let addr = SocketAddr::from(([0, 0, 0, 0], ports.http));
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

    if let Ok(r) = axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
    {
        r
    } else {
        tracing::error!("Failed to start server at 0.0.0.0:{}", ports.https);
    }
}

pub fn create_routes(
    base_path: PathBuf,
    site_graph: Arc<SiteGraph>,
    site_config: Config,
    tera: Arc<Tera>,
    data_path: &Path,
    store_uri: String,
    certs_path: String,
) -> Router {
    let storage_path = data_path.join(kernel::sqlite::DATABASE);
    let sessions_path = data_path.join(SESSIONS_DATABASE);

    let google_authorizer = GoogleAuthorizer::new(storage_path.as_path()).unwrap();
    let github_authorizer = GithubAuthorizer::new(storage_path.as_path()).unwrap();
    let yandex_authorizer = YandexAuthorizer::new(storage_path.as_path()).unwrap();

    let google_authorizer = Arc::new(google_authorizer);
    let github_authorizer = Arc::new(github_authorizer);
    let yandex_authorizer = Arc::new(yandex_authorizer);

    let storage_path_clone = storage_path.clone();
    let user_store = UserStorage::from(Arc::new(storage_path_clone));

    let storage = Sqlite::open(storage_path, Mode::ReadWrite).unwrap();
    let storage = Arc::new(Mutex::new(storage));
    let cache = Arc::new(Mutex::new(HashSet::new()));

    let public_key_path = PathBuf::from(&certs_path)
        .join("egoroffspbrupub.pem")
        .to_str()
        .unwrap()
        .to_string();
    let public_key_path = Arc::new(public_key_path);

    let page_context = Arc::new(PageContext {
        base_path,
        storage,
        tera,
        site_graph,
        site_config,
        store_uri,
        certs_path,
        cache,
    });

    let secret = rand::thread_rng().gen::<[u8; 64]>();
    let session_store = SqliteSessionStore::open(sessions_path, &secret).unwrap();
    session_store.cleanup().unwrap();
    let secret = session_store.get_secret().unwrap();
    let session_layer = SessionLayer::new(session_store, &secret)
        .with_secure(false)
        .with_session_ttl(Some(Duration::from_secs(86400 * 14)))
        .with_same_site_policy(SameSite::Lax)
        .with_persistence_policy(axum_sessions::PersistencePolicy::ExistingOnly);

    let auth_layer = AuthLayer::new(user_store, &secret);

    #[derive(OpenApi)]
    #[openapi(
        paths(
            handlers::blog::serve_posts_api,
            handlers::micropub::serve_index_get,
            handlers::micropub::serve_index_post,
            handlers::indie::serve_token_generate,
            handlers::indie::serve_token_validate,
        ),
        components(
            schemas(SmallPost, kernel::domain::SmallPosts, micropub::MicropubConfig, micropub::SyndicateTo, micropub::MicropubFormError, indie::TokenValidationResult, indie::Token, indie::TokenRequest),
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "egoroff.spb.ru", description = "egoroff.spb.ru API")
        ),
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "authorization",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("authorization"))),
                );
            }
        }
    }

    let login_handler = handlers::auth::serve_login
        .layer(Extension(google_authorizer.clone()))
        .layer(Extension(github_authorizer.clone()))
        .layer(Extension(yandex_authorizer.clone()));

    // build our custom compression predicate
    // its recommended to still include `DefaultPredicate` as part of
    // custom predicates
    let compress_predicate = DefaultPredicate::new()
        .and(NotForContentType::new("text/html"))
        .and(NotForContentType::new("application/octet-stream"))
        .and(NotForContentType::new("application/json"));

    let router = Router::new()
        .route("/auth", get(handlers::indie::serve_auth))
        .route("/admin", get(handlers::admin::serve_admin))
        .route(
            "/api/v2/admin/posts/",
            get(handlers::blog::serve_posts_admin_api),
        )
        .route("/api/v2/admin/post", put(handlers::blog::serve_post_update))
        .route(
            "/api/v2/admin/post/:id",
            delete(handlers::blog::serve_post_delete),
        )
        // Important all admin protected routes must be the first in the list
        .route_layer(RequireAuth::login_with_role(Role::Admin..))
        .route("/profile", get(handlers::auth::serve_profile))
        .route("/profile/", get(handlers::auth::serve_profile))
        .route("/logout", get(handlers::auth::serve_logout))
        .route("/logout/", get(handlers::auth::serve_logout))
        .route(
            "/api/v2/auth/userinfo",
            get(handlers::auth::serve_user_info_api_call),
        )
        .route(
            "/api/v2/auth/userinfo/",
            get(handlers::auth::serve_user_info_api_call),
        )
        // Important all protected routes must be the first in the list
        .route_layer(RequireAuth::login())
        .merge(SwaggerUi::new("/api/v2").url("/api/v2/openapi.json", ApiDoc::openapi()))
        .route(
            "/micropub/",
            get(handlers::micropub::serve_index_get)
                .layer(RequireIndieAuthorizationLayer::auth(
                    public_key_path.clone(),
                ))
                .post(handlers::micropub::serve_index_post)
                .layer(RequireIndieAuthorizationLayer::auth(
                    public_key_path.clone(),
                )),
        )
        .route(
            "/micropub",
            get(handlers::micropub::serve_index_get)
                .layer(RequireIndieAuthorizationLayer::auth(
                    public_key_path.clone(),
                ))
                .post(handlers::micropub::serve_index_post)
                .layer(RequireIndieAuthorizationLayer::auth(public_key_path)),
        )
        .route("/", get(handlers::serve_index))
        .route("/recent.atom", get(handlers::blog::serve_atom))
        .route("/sitemap.xml", get(handlers::serve_sitemap))
        .route("/news/rss", get(handlers::blog::serve_atom))
        .route("/portfolio/", get(handlers::portfolio::serve_index))
        .route(
            "/portfolio/:path",
            get(handlers::portfolio::serve_apache_document),
        )
        .route(
            "/portfolio/apache/:path",
            get(handlers::portfolio::redirect_to_real_document),
        )
        .route(
            "/portfolio/portfolio/:path",
            get(handlers::portfolio::redirect_to_real_document),
        )
        .route("/blog/", get(handlers::blog::serve_index_default))
        .route("/news/", get(handlers::blog::redirect_to_blog))
        .route("/opinions/", get(handlers::blog::serve_index_default))
        .route(
            "/blog/page/:page",
            get(handlers::blog::serve_index_not_default),
        )
        .route(
            "/blog/page/:page/",
            get(handlers::blog::serve_index_not_default),
        )
        .route("/blog/recent.atom", get(handlers::blog::serve_atom))
        .route("/blog/:path", get(handlers::blog::serve_document))
        .route(
            "/opinions/:path",
            get(handlers::blog::redirect_to_real_document),
        )
        .route("/search/", get(handlers::serve_search))
        .route("/:path", get(handlers::serve_root))
        .route("/js/:path", get(handlers::serve_js))
        .route("/css/:path", get(handlers::serve_css))
        .route("/img/:path", get(handlers::serve_img))
        .route("/apache/:path", get(handlers::serve_apache))
        .route("/apache/images/:path", get(handlers::serve_apache_images))
        .route("/login", get(login_handler))
        .route(
            "/_s/callback/google/authorized/",
            get(handlers::auth::google_oauth_callback).layer(Extension(google_authorizer)),
        )
        .route(
            "/_s/callback/github/authorized/",
            get(handlers::auth::github_oauth_callback).layer(Extension(github_authorizer)),
        )
        .route(
            "/_s/callback/yandex/authorized/",
            get(handlers::auth::yandex_oauth_callback).layer(Extension(yandex_authorizer)),
        )
        .route("/api/v2/navigation/", get(handlers::serve_navigation))
        .route(
            "/api/v2/blog/archive/",
            get(handlers::blog::serve_archive_api),
        )
        .route("/api/v2/blog/posts/", get(handlers::blog::serve_posts_api))
        .route(
            "/api/v2/auth/user/",
            get(handlers::auth::serve_user_api_call),
        )
        .route(
            "/api/v2/auth/user",
            get(handlers::auth::serve_user_api_call),
        )
        .route("/storage/:bucket/:path", get(handlers::serve_storage))
        .route(
            "/token",
            post(handlers::indie::serve_token_generate).get(handlers::indie::serve_token_validate),
        )
        .route(
            "/token/",
            post(handlers::indie::serve_token_generate).get(handlers::indie::serve_token_validate),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http().on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        tracing::error!("Server error: {error}");
                    },
                ))
                .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
                .layer(session_layer)
                .layer(auth_layer)
                .into_inner(),
        )
        .layer(CompressionLayer::new().compress_when(compress_predicate))
        .layer(RequestBodyLimitLayer::new(20 * 1024 * 1024))
        .layer(DefaultBodyLimit::disable())
        .with_state(page_context);

    #[cfg(feature = "prometheus")]
    return router.layer(prometheus_layer);

    #[cfg(not(feature = "prometheus"))]
    return router;
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
