use super::{template::Profile, *};
use crate::auth::AppUser;
use crate::{
    auth::{ToUser, YandexAuthorizer},
    body::Redirect,
    domain::AuthorizedUser,
    handlers::template::Signin,
};
use oauth2::{CsrfToken, PkceCodeVerifier, TokenResponse};
use serde::Deserialize;
use tower_sessions::Session;
use url::Url;

use crate::{
    auth::{AuthBackend, Authorizer, GithubAuthorizer, GoogleAuthorizer},
    domain::AuthRequest,
};

type AuthSession = axum_login::AuthSession<AuthBackend>;

const GOOGLE_CSRF_KEY: &str = "google_csrf_state";
const GITHUB_CSRF_KEY: &str = "github_csrf_state";
const YANDEX_CSRF_KEY: &str = "yandex_csrf_state";
const PKCE_CODE_VERIFIER_KEY_GOOGLE: &str = "google_pkce_code_verifier";
const PKCE_CODE_VERIFIER_KEY_GITHUB: &str = "github_pkce_code_verifier";
const PKCE_CODE_VERIFIER_KEY_YANDEX: &str = "yandex_pkce_code_verifier";
const REDIRECT_TO_KEY: &str = "redirect_uri";
pub const PROFILE_URI: &str = "/profile/";
pub const LOGIN_URI: &str = "/login";

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub next: Option<String>,
}

macro_rules! register_url {
    ($context:ident, $session:ident, $url:ident, $key:ident, $context_param:ident) => {{
        $context.$context_param = $url.url.as_str();
        let r = $session.insert($key, $url.csrf_state).await;
        if let Err(e) = r {
            tracing::error!("error register url: {e:#?}");
        };
    }};
}

/// Extract the `redirect_uri` query parameter from a `next` URL string.
/// The `next` parameter is expected to be a URL like `/auth?me=...&redirect_uri=...`
fn extract_redirect_uri_from_next(next: &str) -> Option<String> {
    // Try to parse as a URL
    let uri = if let Ok(parsed) = Url::parse(next) {
        // If parsing succeeds (absolute URL), extract redirect_uri
        parsed
    } else {
        // If parsing fails, try to parse as a relative URL by adding a dummy base
        let base = Url::parse("http://dummy.example").ok()?;
        base.join(next).ok()?
    };
    uri.query_pairs()
        .find(|(key, _)| key == "redirect_uri")
        .map(|(_, value)| value.into_owned())
}

pub async fn serve_login(
    Query(query): Query<LoginQuery>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    Extension(gitgub_authorizer): Extension<Arc<GithubAuthorizer>>,
    Extension(yandex_authorizer): Extension<Arc<YandexAuthorizer>>,
    session: Session,
) -> impl IntoResponse {
    if let Some(next) = query.next {
        // Try to parse the next URL to extract redirect_uri parameter
        let extracted_redirect_uri = extract_redirect_uri_from_next(&next);
        if let Some(redirect_uri) = &extracted_redirect_uri {
            tracing::debug!("Extracted redirect_uri from next parameter: {redirect_uri}");
            if let Err(e) = session.insert(REDIRECT_TO_KEY, redirect_uri).await {
                tracing::error!("error inserting extracted redirect_uri: {e:#?}");
            }
        } else {
            tracing::debug!("No redirect_uri found in next parameter");
        }
    } else {
        tracing::debug!("Next parameter not found");
    }

    let mut context = Signin {
        year: get_year(),
        ..Default::default()
    };
    let google_url = google_authorizer.generate_authorize_url();
    let github_url = gitgub_authorizer.generate_authorize_url();
    let yandex_url = yandex_authorizer.generate_authorize_url();

    register_url!(
        context,
        session,
        google_url,
        GOOGLE_CSRF_KEY,
        google_signin_url
    );

    if let Some(v) = google_url.verifier
        && let Err(e) = session.insert(PKCE_CODE_VERIFIER_KEY_GOOGLE, v).await
    {
        tracing::error!("error inserting Google pkce_code_verifier: {e:#?}");
    }

    register_url!(
        context,
        session,
        github_url,
        GITHUB_CSRF_KEY,
        github_signin_url
    );

    if let Some(v) = github_url.verifier
        && let Err(e) = session.insert(PKCE_CODE_VERIFIER_KEY_GITHUB, v).await
    {
        tracing::error!("error inserting GitHub pkce_code_verifier: {e:#?}");
    }

    register_url!(
        context,
        session,
        yandex_url,
        YANDEX_CSRF_KEY,
        yandex_signin_url
    );

    if let Some(v) = yandex_url.verifier
        && let Err(e) = session.insert(PKCE_CODE_VERIFIER_KEY_YANDEX, v).await
    {
        tracing::error!("error inserting Yandex pkce_code_verifier: {e:#?}");
    }

    context.title = "Авторизация";

    context.into_response()
}

pub async fn serve_logout(mut auth: AuthSession) -> impl IntoResponse {
    if let Err(e) = auth.logout().await {
        tracing::error!("Logout failed: {e:#?}");
    }
    Redirect::to(LOGIN_URI)
}

pub async fn serve_profile() -> impl IntoResponse {
    Profile {
        title: "Редактирование профиля",
        year: get_year(),
        ..Default::default()
    }
    .into_response()
}

macro_rules! login_user_using_token {
    ($token:expr, $session:ident, $storage:ident, $auth:ident, $authorizer:ident) => {{
        match $token {
            Ok(token) => {
                let user = $authorizer.get_user(token.access_token()).await;
                match user {
                    Ok(u) => {
                        drop($session);
                        let user = u.to_user();
                        if let Err(e) = $storage.upsert_user(&user) {
                            tracing::error!("login error: {e:#?}");
                            return Redirect::to(LOGIN_URI);
                        }
                        tracing::info!("User updated");

                        let u = AppUser::new(user);
                        match $auth.login(&u).await {
                            Ok(()) => tracing::info!("login success"),
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
        match $session.get::<CsrfToken>($key).await {
            Ok(original_csrf_state) => {
                if let Some(state) = original_csrf_state {
                    if state.secret() == $query.state.secret() {
                        tracing::info!("authorized");
                    } else {
                        tracing::error!("unauthorized: secret mismatch");
                        return Redirect::to(LOGIN_URI);
                    }
                } else {
                    tracing::error!("unauthorized: no original_csrf_state");
                    return Redirect::to(LOGIN_URI);
                }
            }
            Err(e) => {
                tracing::error!("No state from session: {}", e);
                return Redirect::to(LOGIN_URI);
            }
        }
    }};
}

pub async fn google_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    State(page_context): State<Arc<PageContext<'_>>>,
    session: Session,
    mut auth: AuthSession,
) -> impl IntoResponse {
    validate_csrf!(GOOGLE_CSRF_KEY, session, query);

    // Get redirect_to from session before it might be dropped
    let redirect_to = session.get::<String>(REDIRECT_TO_KEY).await.unwrap_or(None);

    if let Ok(pkce_code_verifier) = session
        .get::<PkceCodeVerifier>(PKCE_CODE_VERIFIER_KEY_GOOGLE)
        .await
    {
        let token = google_authorizer
            .exchange_code(query.code, pkce_code_verifier)
            .await;
        let mut storage = page_context.storage.lock().await;
        login_user_using_token!(token, session, storage, auth, google_authorizer);

        if let Some(redirect_url) = &redirect_to {
            tracing::debug!("OAuth callback redirecting to: {}", redirect_url);
            Redirect::to(redirect_url)
        } else {
            tracing::debug!("No redirect destination found, redirecting to profile");
            Redirect::to(PROFILE_URI)
        }
    } else {
        tracing::error!("No code verifier from session");
        Redirect::to(LOGIN_URI)
    }
}

pub async fn github_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(github_authorizer): Extension<Arc<GithubAuthorizer>>,
    State(page_context): State<Arc<PageContext<'_>>>,
    session: Session,
    mut auth: AuthSession,
) -> impl IntoResponse {
    validate_csrf!(GITHUB_CSRF_KEY, session, query);

    // Get redirect_to from session before it might be dropped
    let redirect_to = session.get::<String>(REDIRECT_TO_KEY).await.unwrap_or(None);

    if let Ok(pkce_code_verifier) = session
        .get::<PkceCodeVerifier>(PKCE_CODE_VERIFIER_KEY_GITHUB)
        .await
    {
        let token = github_authorizer
            .exchange_code(query.code, pkce_code_verifier)
            .await;
        let mut storage = page_context.storage.lock().await;
        login_user_using_token!(token, session, storage, auth, github_authorizer);

        if let Some(redirect_url) = redirect_to {
            tracing::debug!("OAuth callback redirecting to: {}", redirect_url);
            Redirect::to(&redirect_url)
        } else {
            tracing::debug!("No redirect destination found, redirecting to profile");
            Redirect::to(PROFILE_URI)
        }
    } else {
        tracing::error!("No code verifier from session");
        Redirect::to(LOGIN_URI)
    }
}

pub async fn yandex_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(yandex_authorizer): Extension<Arc<YandexAuthorizer>>,
    State(page_context): State<Arc<PageContext<'_>>>,
    session: Session,
    mut auth: AuthSession,
) -> impl IntoResponse {
    validate_csrf!(YANDEX_CSRF_KEY, session, query);

    // Get redirect_to from session before it might be dropped
    let redirect_to = session.get::<String>(REDIRECT_TO_KEY).await.unwrap_or(None);

    if let Ok(pkce_code_verifier) = session
        .get::<PkceCodeVerifier>(PKCE_CODE_VERIFIER_KEY_YANDEX)
        .await
    {
        let token = yandex_authorizer
            .exchange_code(query.code, pkce_code_verifier)
            .await;
        let mut storage = page_context.storage.lock().await;
        login_user_using_token!(token, session, storage, auth, yandex_authorizer);

        if let Some(redirect_url) = redirect_to {
            tracing::debug!("OAuth callback redirecting to: {}", redirect_url);
            Redirect::to(&redirect_url)
        } else {
            tracing::debug!("No redirect destination found, redirecting to profile");
            Redirect::to(PROFILE_URI)
        }
    } else {
        tracing::error!("No code verifier from session");
        Redirect::to(LOGIN_URI)
    }
}

pub async fn serve_user_api_call(auth: AuthSession) -> impl IntoResponse {
    match auth.user {
        Some(user) => Json(user.into_authorized()),
        None => Json(AuthorizedUser::default()),
    }
}

pub async fn serve_user_info_api_call(auth: AuthSession) -> impl IntoResponse {
    if let Some(u) = auth.user {
        u.into_response()
    } else {
        Redirect::to(LOGIN_URI).into_response()
    }
}
