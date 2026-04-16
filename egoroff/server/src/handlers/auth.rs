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

async fn register_auth_url(
    session: &Session,
    csrf_key: &'static str,
    pkce_key: &'static str,
    csrf_token: CsrfToken,
    verifier: PkceCodeVerifier,
) {
    if let Err(e) = session.insert(csrf_key, csrf_token).await {
        tracing::error!("error register url (csrf): {e:#?}");
    }

    if let Err(e) = session.insert(pkce_key, verifier).await {
        tracing::error!("error inserting {} pkce_code_verifier: {e:#?}", pkce_key);
    }
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

    let google_url = google_authorizer.generate_authorize_url();
    let github_url = gitgub_authorizer.generate_authorize_url();
    let yandex_url = yandex_authorizer.generate_authorize_url();

    register_auth_url(
        &session,
        GOOGLE_CSRF_KEY,
        PKCE_CODE_VERIFIER_KEY_GOOGLE,
        google_url.csrf_state,
        google_url.verifier,
    )
    .await;

    register_auth_url(
        &session,
        GITHUB_CSRF_KEY,
        PKCE_CODE_VERIFIER_KEY_GITHUB,
        github_url.csrf_state,
        github_url.verifier,
    )
    .await;

    register_auth_url(
        &session,
        YANDEX_CSRF_KEY,
        PKCE_CODE_VERIFIER_KEY_YANDEX,
        yandex_url.csrf_state,
        yandex_url.verifier,
    )
    .await;

    let context = Signin {
        year: get_year(),
        google_signin_url: google_url.url.as_str(),
        github_signin_url: github_url.url.as_str(),
        yandex_signin_url: yandex_url.url.as_str(),
        title: "Авторизация",
        ..Default::default()
    };

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

async fn oauth_callback<T: auth::ToUser, U: Authorizer<T>>(
    query: AuthRequest,
    authorizer: Arc<U>,
    page_context: Arc<PageContext<'static>>,
    session: Session,
    pkce_code_verifier_key: &str,
    csrf_key: &str,
    mut auth: AuthSession,
) -> impl IntoResponse {
    match session.get::<CsrfToken>(csrf_key).await {
        Ok(original_csrf_state) => {
            if let Some(state) = original_csrf_state {
                if state.secret() == query.state.secret() {
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

    // Get redirect_to from session before it might be dropped
    let redirect_to = session.get::<String>(REDIRECT_TO_KEY).await.unwrap_or(None);

    if let Ok(pkce_code_verifier) = session
        .get::<PkceCodeVerifier>(pkce_code_verifier_key)
        .await
        && let Some(pkce_code_verifier) = pkce_code_verifier
    {
        let token = authorizer
            .exchange_code(query.code.clone(), pkce_code_verifier)
            .await;
        let mut storage = page_context.storage.lock().await;

        match token {
            Ok(token) => {
                let user = authorizer.get_user(token.access_token()).await;
                match user {
                    Ok(u) => {
                        drop(session);
                        let user = u.to_user();
                        if let Err(e) = storage.upsert_user(&user) {
                            tracing::error!("login error: {e:#?}");
                            return Redirect::to(LOGIN_URI);
                        }
                        tracing::info!("User updated");

                        let u = AppUser::new(user);
                        match auth.login(&u).await {
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

#[axum::debug_handler]
pub async fn google_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    State(page_context): State<Arc<PageContext<'static>>>,
    session: Session,
    auth: AuthSession,
) -> impl IntoResponse {
    oauth_callback(
        query,
        google_authorizer,
        page_context,
        session,
        PKCE_CODE_VERIFIER_KEY_GOOGLE,
        GOOGLE_CSRF_KEY,
        auth,
    )
    .await
}

#[axum::debug_handler]
pub async fn github_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(github_authorizer): Extension<Arc<GithubAuthorizer>>,
    State(page_context): State<Arc<PageContext<'static>>>,
    session: Session,
    auth: AuthSession,
) -> impl IntoResponse {
    oauth_callback(
        query,
        github_authorizer,
        page_context,
        session,
        PKCE_CODE_VERIFIER_KEY_GITHUB,
        GITHUB_CSRF_KEY,
        auth,
    )
    .await
}

#[axum::debug_handler]
pub async fn yandex_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(yandex_authorizer): Extension<Arc<YandexAuthorizer>>,
    State(page_context): State<Arc<PageContext<'static>>>,
    session: Session,
    auth: AuthSession,
) -> impl IntoResponse {
    oauth_callback(
        query,
        yandex_authorizer,
        page_context,
        session,
        PKCE_CODE_VERIFIER_KEY_YANDEX,
        YANDEX_CSRF_KEY,
        auth,
    )
    .await
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
