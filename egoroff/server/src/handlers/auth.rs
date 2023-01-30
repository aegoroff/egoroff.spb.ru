use super::*;
use crate::{
    auth::{ToUser, YandexAuthorizer},
    body::Redirect,
    domain::AuthorizedUser,
};
use oauth2::{CsrfToken, PkceCodeVerifier, TokenResponse};

use crate::{
    auth::{Authorizer, GithubAuthorizer, GoogleAuthorizer, Role, UserStorage},
    domain::AuthRequest,
};

type AuthContext = axum_login::extractors::AuthContext<User, UserStorage, Role>;

const GOOGLE_CSRF_KEY: &str = "google_csrf_state";
const GITHUB_CSRF_KEY: &str = "github_csrf_state";
const YANDEX_CSRF_KEY: &str = "yandex_csrf_state";
const PKCE_CODE_VERIFIER_KEY: &str = "pkce_code_verifier";
const PROFILE_URI: &str = "/profile/";
const LOGIN_URI: &str = "/login";

macro_rules! register_url {
    ($context:ident, $session:ident, $url:ident, $key:ident, $context_param:expr) => {{
        $session.insert($key, $url.csrf_state).unwrap();
        $context.insert($context_param, $url.url.as_str());
    }};
}

pub async fn serve_login(
    State(page_context): State<Arc<PageContext>>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    Extension(gitgub_authorizer): Extension<Arc<GithubAuthorizer>>,
    Extension(yandex_authorizer): Extension<Arc<YandexAuthorizer>>,
    mut session: WritableSession,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(CONFIG_KEY, &page_context.site_config);

    let google_url = google_authorizer.generate_authorize_url();
    let github_url = gitgub_authorizer.generate_authorize_url();
    let yandex_url = yandex_authorizer.generate_authorize_url();

    register_url!(
        context,
        session,
        google_url,
        GOOGLE_CSRF_KEY,
        "google_signin_url"
    );

    session
        .insert(PKCE_CODE_VERIFIER_KEY, google_url.verifier.unwrap())
        .unwrap();

    register_url!(
        context,
        session,
        github_url,
        GITHUB_CSRF_KEY,
        "github_signin_url"
    );
    register_url!(
        context,
        session,
        yandex_url,
        YANDEX_CSRF_KEY,
        "yandex_signin_url"
    );

    context.insert(TITLE_KEY, "Авторизация");

    serve_page(&context, "signin.html", &page_context.tera)
}

pub async fn serve_logout(mut auth: AuthContext) -> impl IntoResponse {
    auth.logout().await;
    Redirect::to("/login")
}

pub async fn serve_profile(State(page_context): State<Arc<PageContext>>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(CONFIG_KEY, &page_context.site_config);
    context.insert(TITLE_KEY, "Редактирование профиля");

    serve_page(&context, "profile.html", &page_context.tera)
}

macro_rules! login_user_using_token {
    ($token:expr, $session:ident, $storage:ident, $auth:ident, $type:tt) => {{
        match $token {
            Ok(token) => {
                let user = $type::get_user(token.access_token()).await;
                match user {
                    Ok(u) => {
                        drop($session);
                        let user = u.to_user();
                        if let Err(e) = $storage.upsert_user(&user) {
                            tracing::error!("login error: {e:#?}");
                            return Redirect::to(LOGIN_URI);
                        }
                        tracing::info!("User updated");

                        match $auth.login(&user).await {
                            Ok(_) => tracing::info!("login success"),
                            Err(e) => {
                                tracing::error!("login error: {e:#?}");
                                return Redirect::to(LOGIN_URI);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("get user error: {e:#?}");
                        return Redirect::to(LOGIN_URI);
                    }
                }
            }
            Err(e) => {
                tracing::error!("token error: {e:#?}");
                return Redirect::to(LOGIN_URI);
            }
        }
    }};
}

macro_rules! validate_csrf {
    ($key:expr, $session:ident, $query:ident) => {{
        match $session.get::<CsrfToken>($key) {
            Some(original_csrf_state) => {
                if original_csrf_state.secret() == $query.state.secret() {
                    tracing::info!("authorized");
                } else {
                    tracing::error!("unauthorized");
                    return Redirect::to(LOGIN_URI);
                }
            }
            None => {
                tracing::error!("No state from session");
                return Redirect::to(LOGIN_URI);
            }
        }
    }};
}

pub async fn google_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    State(page_context): State<Arc<PageContext>>,
    session: ReadableSession,
    mut auth: AuthContext,
) -> impl IntoResponse {
    validate_csrf!(GOOGLE_CSRF_KEY, session, query);
    match session.get::<PkceCodeVerifier>(PKCE_CODE_VERIFIER_KEY) {
        Some(pkce_code_verifier) => {
            let token = google_authorizer
                .exchange_code(query.code, Some(pkce_code_verifier))
                .await;
            let mut storage = page_context.storage.lock().await;
            login_user_using_token!(token, session, storage, auth, GoogleAuthorizer);
        }
        None => {
            tracing::error!("No code verifier from session");
            return Redirect::to(LOGIN_URI);
        }
    }

    Redirect::to(PROFILE_URI)
}

pub async fn github_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(github_authorizer): Extension<Arc<GithubAuthorizer>>,
    State(page_context): State<Arc<PageContext>>,
    session: ReadableSession,
    mut auth: AuthContext,
) -> impl IntoResponse {
    validate_csrf!(GITHUB_CSRF_KEY, session, query);
    let token = github_authorizer.exchange_code(query.code, None).await;
    let mut storage = page_context.storage.lock().await;
    login_user_using_token!(token, session, storage, auth, GithubAuthorizer);
    Redirect::to(PROFILE_URI)
}

pub async fn yandex_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(yandex_authorizer): Extension<Arc<YandexAuthorizer>>,
    State(page_context): State<Arc<PageContext>>,
    session: ReadableSession,
    mut auth: AuthContext,
) -> impl IntoResponse {
    validate_csrf!(YANDEX_CSRF_KEY, session, query);
    let token = yandex_authorizer.exchange_code(query.code, None).await;
    let mut storage = page_context.storage.lock().await;
    login_user_using_token!(token, session, storage, auth, YandexAuthorizer);
    Redirect::to(PROFILE_URI)
}

pub async fn serve_user_api_call(auth: AuthContext) -> impl IntoResponse {
    match auth.current_user {
        Some(user) => {
            let authenticated = AuthorizedUser {
                login_or_name: user.login,
                authenticated: true,
                admin: user.admin,
                provider: user.provider,
            };
            Json(authenticated)
        }
        None => Json(AuthorizedUser {
            ..Default::default()
        }),
    }
}

pub async fn serve_user_info_api_call(Extension(user): Extension<User>) -> impl IntoResponse {
    Json(user)
}
