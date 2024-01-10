use crate::auth::{AuthBackend, GithubAuthorizer, GoogleAuthorizer, Role, YandexAuthorizer};
use anyhow::Result;
use axum::extract::DefaultBodyLimit;
use axum::handler::Handler;
use axum::routing::{delete, post, put};
use axum::Extension;
use axum::{routing::get, Router};

use axum_login::{login_required, permission_required, AuthManagerLayerBuilder};
use tower_sessions::cookie::{time::Duration, SameSite};
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::domain::PageContext;
use futures::lock::Mutex;
use indie::RequireIndieAuthorizationLayer;
use kernel::domain::SmallPost;
use kernel::graph::SiteGraph;
use kernel::session::SqliteSessionStore;
use kernel::sqlite::{Mode, Sqlite};
use rand::Rng;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::compression::predicate::NotForContentType;
use tower_http::compression::{CompressionLayer, DefaultPredicate, Predicate};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::domain::Config;
use crate::{handlers, indie, micropub};

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

#[derive(OpenApi)]
#[openapi(
        paths(
            handlers::blog::serve_posts_api,
            handlers::micropub::serve_index_get,
            handlers::micropub::serve_index_post,
            handlers::micropub::serve_media_endpoint_post,
            handlers::micropub::serve_media_endpoint_get,
            handlers::indie::serve_token_generate,
            handlers::indie::serve_token_validate,
        ),
        components(
            schemas(SmallPost, kernel::domain::SmallPosts, micropub::MicropubConfig, micropub::SyndicateTo, micropub::MicropubFormError, indie::TokenValidationResult, indie::Token, indie::TokenRequest, handlers::micropub::MediaResponse),
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "egoroff.spb.ru", description = "egoroff.spb.ru API")
        ),
    )]
struct ApiDoc;

pub fn create_routes(
    base_path: PathBuf,
    site_graph: Arc<SiteGraph<'static>>,
    site_config: Config,
    data_path: &Path,
    store_uri: String,
    certs_path: String,
) -> Result<Router> {
    let storage_path = data_path.join(kernel::sqlite::DATABASE);
    let sessions_path = data_path.join(crate::SESSIONS_DATABASE);

    let storage_path_clone = storage_path.clone();
    let auth_backend = AuthBackend::from(Arc::new(storage_path_clone));

    let storage = Sqlite::open(storage_path.clone(), Mode::ReadWrite)?;
    let storage = Arc::new(Mutex::new(storage));
    let cache = Arc::new(Mutex::new(HashSet::new()));

    let public_key_path = PathBuf::from(&certs_path)
        .join("egoroffspbrupub.pem")
        .to_str()
        .unwrap_or_default()
        .to_string();
    let public_key_path = Arc::new(public_key_path);

    let page_context = Arc::new(PageContext {
        base_path,
        storage,
        site_graph,
        site_config,
        store_uri,
        certs_path,
        cache,
    });

    let secret = rand::thread_rng().gen::<[u8; 64]>();
    let session_store = SqliteSessionStore::open(sessions_path, &secret)?;
    session_store.cleanup()?;
    let session_expiry = Expiry::OnInactivity(Duration::seconds(86400 * 14));
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(session_expiry)
        .with_same_site(SameSite::Lax);

    let session_service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http().on_failure(
            |error: ServerErrorsFailureClass, _latency: std::time::Duration, _span: &Span| {
                tracing::error!("Server error: {error}");
            },
        ))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
        .layer(AuthManagerLayerBuilder::new(auth_backend, session_layer).build())
        .into_inner();

    // build our custom compression predicate
    // its recommended to still include `DefaultPredicate` as part of
    // custom predicates
    let compress_predicate = DefaultPredicate::new()
        .and(NotForContentType::new("text/html"))
        .and(NotForContentType::new("application/octet-stream"))
        .and(NotForContentType::new("application/json"));

    let router = Router::new()
        .route("/auth", get(handlers::indie::serve_auth))
        .route("/admin", get(handlers::admin::serve))
        .nest("/api/v2/admin", admin_api())
        // Important all admin protected routes must be the first in the list
        .route_layer(permission_required!(
            AuthBackend,
            login_url = handlers::auth::LOGIN_URI,
            Role::Admin
        ))
        .route("/profile", get(handlers::auth::serve_profile))
        .route(
            handlers::auth::PROFILE_URI,
            get(handlers::auth::serve_profile),
        )
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
        .route_layer(login_required!(
            AuthBackend,
            login_url = handlers::auth::LOGIN_URI
        ))
        .merge(SwaggerUi::new("/api/v2").url("/api/v2/openapi.json", ApiDoc::openapi()))
        .route(
            "/micropub/",
            get(handlers::micropub::serve_index_get)
                .post(handlers::micropub::serve_index_post)
                .layer(RequireIndieAuthorizationLayer::auth(
                    public_key_path.clone(),
                )),
        )
        .route(
            "/micropub",
            get(handlers::micropub::serve_index_get)
                .post(handlers::micropub::serve_index_post)
                .layer(RequireIndieAuthorizationLayer::auth(
                    public_key_path.clone(),
                )),
        )
        .route(
            "/micropub/media",
            get(handlers::micropub::serve_media_endpoint_get)
                .post(handlers::micropub::serve_media_endpoint_post)
                .layer(RequireIndieAuthorizationLayer::auth(public_key_path)),
        )
        .route("/", get(handlers::serve_index))
        .route("/recent.atom", get(handlers::blog::serve_atom))
        .route("/sitemap.xml", get(handlers::serve_sitemap))
        .route("/news/rss", get(handlers::blog::serve_atom))
        .route("/news/rss/", get(handlers::blog::serve_atom))
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
        .route("/news/", get(handlers::blog::redirect))
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
        .nest("/api/v2", public_api())
        .merge(oauth2_routes(storage_path)?)
        .route("/storage/:bucket/:path", get(handlers::serve_storage))
        .route(
            "/token",
            post(handlers::indie::serve_token_generate).get(handlers::indie::serve_token_validate),
        )
        .route(
            "/token/",
            post(handlers::indie::serve_token_generate).get(handlers::indie::serve_token_validate),
        )
        .layer(session_service)
        .layer(CompressionLayer::new().compress_when(compress_predicate))
        .layer(RequestBodyLimitLayer::new(20 * 1024 * 1024))
        .layer(DefaultBodyLimit::disable())
        .with_state(page_context);

    #[cfg(feature = "prometheus")]
    return router.layer(prometheus_layer);

    #[cfg(not(feature = "prometheus"))]
    return Ok(router);
}

fn admin_api() -> Router<Arc<PageContext<'static>>> {
    Router::new()
        .route("/posts/", get(handlers::blog::serve_posts_admin_api))
        .route("/post", put(handlers::blog::serve_post_update))
        .route("/post/:id", delete(handlers::blog::serve_post_delete))
}

fn public_api() -> Router<Arc<PageContext<'static>>> {
    Router::new()
        .route("/navigation/", get(handlers::serve_navigation))
        .route("/blog/archive/", get(handlers::blog::serve_archive_api))
        .route("/blog/posts/", get(handlers::blog::serve_posts_api))
        .route("/auth/user/", get(handlers::auth::serve_user_api_call))
        .route("/auth/user", get(handlers::auth::serve_user_api_call))
}

fn oauth2_routes(storage_path: PathBuf) -> Result<Router<Arc<PageContext<'static>>>> {
    let google_authorizer = GoogleAuthorizer::new(storage_path.as_path())?;
    let github_authorizer = GithubAuthorizer::new(storage_path.as_path())?;
    let yandex_authorizer = YandexAuthorizer::new(storage_path.as_path())?;

    let google_authorizer = Arc::new(google_authorizer);
    let github_authorizer = Arc::new(github_authorizer);
    let yandex_authorizer = Arc::new(yandex_authorizer);

    let login_handler = handlers::auth::serve_login
        .layer(Extension(google_authorizer.clone()))
        .layer(Extension(github_authorizer.clone()))
        .layer(Extension(yandex_authorizer.clone()));

    let callbacks = Router::new()
        .route(
            "/google/authorized/",
            get(handlers::auth::google_oauth_callback).layer(Extension(google_authorizer)),
        )
        .route(
            "/github/authorized/",
            get(handlers::auth::github_oauth_callback).layer(Extension(github_authorizer)),
        )
        .route(
            "/yandex/authorized/",
            get(handlers::auth::yandex_oauth_callback).layer(Extension(yandex_authorizer)),
        );
    Ok(Router::new()
        .route(handlers::auth::LOGIN_URI, get(login_handler))
        .nest("/_s/callback", callbacks))
}
