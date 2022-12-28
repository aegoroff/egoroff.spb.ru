use super::*;
use crate::{auth::ToUser, domain::AuthorizedUser};
use oauth2::{CsrfToken, PkceCodeVerifier, TokenResponse};

use crate::{
    auth::{Authorizer, GithubAuthorizer, GoogleAuthorizer, Role, UserStorage},
    domain::AuthRequest,
};

type AuthContext = axum_login::extractors::AuthContext<User, UserStorage, Role>;

pub async fn serve_login(
    Extension(page_context): Extension<Arc<PageContext>>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    Extension(gitgub_authorizer): Extension<Arc<GithubAuthorizer>>,
    mut session: WritableSession,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(CONFIG_KEY, &page_context.site_config);

    let google_url = google_authorizer.generate_authorize_url();
    let github_url = gitgub_authorizer.generate_authorize_url();

    session
        .insert("google_csrf_state", google_url.csrf_state)
        .unwrap();
    session
        .insert("github_csrf_state", github_url.csrf_state)
        .unwrap();
    session
        .insert("pkce_code_verifier", google_url.verifier.unwrap())
        .unwrap();

    context.insert(TITLE_KEY, "Авторизация");
    context.insert("google_signin_url", google_url.url.as_str());
    context.insert("github_signin_url", github_url.url.as_str());

    serve_page(&context, "signin.html", &page_context.tera)
}

pub async fn serve_logout(mut auth: AuthContext) -> impl IntoResponse {
    auth.logout().await;
    Redirect::to("/login")
}

pub async fn serve_profile(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(CONFIG_KEY, &page_context.site_config);
    context.insert(TITLE_KEY, "Редактирование профиля");

    serve_page(&context, "profile.html", &page_context.tera)
}

macro_rules! login_user {
    ($user:ident, $session:ident, $page_context:ident, $auth:ident) => {{
        match $user {
            Ok(u) => {
                drop($session);
                let login_result = login(u, $page_context, $auth).await;
                match login_result {
                    Ok(_) => tracing::info!("login success"),
                    Err(e) => {
                        tracing::error!("login error: {e:#?}");
                        return Redirect::to("/login");
                    }
                }
            }
            Err(e) => {
                tracing::error!("get user error: {e:#?}");
                return Redirect::to("/login");
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
                    return Redirect::to("/login");
                }
            }
            None => {
                tracing::error!("No state from session");
                return Redirect::to("/login");
            }
        }
    }};
}

pub async fn google_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    Extension(page_context): Extension<Arc<PageContext>>,
    session: ReadableSession,
    auth: AuthContext,
) -> impl IntoResponse {
    validate_csrf!("google_csrf_state", session, query);
    match session.get::<PkceCodeVerifier>("pkce_code_verifier") {
        Some(pkce_code_verifier) => {
            let token = google_authorizer
                .exchange_code(query.code, Some(pkce_code_verifier))
                .await;
            match token {
                Ok(token) => {
                    let user = GoogleAuthorizer::get_user(token.access_token()).await;
                    login_user!(user, session, page_context, auth);
                }
                Err(e) => {
                    tracing::error!("token error: {e:#?}");
                    return Redirect::to("/login");
                }
            }
        }
        None => {
            tracing::error!("No code verifier from session");
            return Redirect::to("/login");
        }
    }

    Redirect::to("/profile/")
}

pub async fn github_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(github_authorizer): Extension<Arc<GithubAuthorizer>>,
    Extension(page_context): Extension<Arc<PageContext>>,
    session: ReadableSession,
    auth: AuthContext,
) -> impl IntoResponse {
    validate_csrf!("github_csrf_state", session, query);
    let token = github_authorizer.exchange_code(query.code, None).await;
    match token {
        Ok(token) => {
            let user = GithubAuthorizer::get_user(token.access_token()).await;
            login_user!(user, session, page_context, auth);
        }
        Err(e) => {
            tracing::error!("token error: {e:#?}");
            return Redirect::to("/login");
        }
    }

    Redirect::to("/profile/")
}

async fn login<U: ToUser>(
    u: U,
    page_context: Arc<PageContext>,
    mut auth: AuthContext,
) -> Result<()> {
    let user = u.to_user();
    let mut storage = Sqlite::open(&page_context.storage_path, Mode::ReadWrite)?;
    storage.upsert_user(&user).unwrap_or(());
    tracing::info!("User updated");

    let login = auth.login(&user).await;
    match login {
        Ok(_) => tracing::info!("Login success"),
        Err(e) => tracing::error!("login failed: {e:#?}"),
    }
    Ok(())
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
