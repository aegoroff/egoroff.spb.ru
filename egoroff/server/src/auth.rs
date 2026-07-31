#![allow(clippy::module_name_repetitions)]

use std::marker::PhantomData;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use axum::{Json, response};
use axum_login::{AuthUser, AuthnBackend, AuthzBackend};
use chrono::Utc;
use kernel::{
    domain::{OAuthProvider, Storage, User},
    sqlite::{Mode, Sqlite},
};
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    StandardTokenResponse, TokenUrl,
    basic::{BasicClient, BasicTokenType},
    url::Url,
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, de::DeserializeOwned};
use thiserror::Error;

use crate::domain::AuthorizedUser;

type SpecialClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;

#[derive(Clone, Debug)]
pub struct AppUser {
    user: User,
}

impl AppUser {
    pub fn new(user: User) -> Self {
        Self { user }
    }

    pub fn into_authorized(self) -> AuthorizedUser {
        AuthorizedUser {
            login_or_name: self.user.login,
            authenticated: true,
            admin: self.user.admin,
            provider: self.user.provider,
        }
    }
}

impl response::IntoResponse for AppUser {
    fn into_response(self) -> response::Response {
        Json(self.user).into_response()
    }
}

pub struct OAuthAuthorizer<T> {
    client: SpecialClient,
    provider: OAuthProvider,
    _phantom: PhantomData<T>,
}

pub type GoogleAuthorizer = OAuthAuthorizer<GoogleUser>;
pub type GithubAuthorizer = OAuthAuthorizer<GithubUser>;
pub type YandexAuthorizer = OAuthAuthorizer<YandexUser>;

#[derive(Clone)]
pub struct AuthBackend {
    db_path: PathBuf,
}

impl AuthBackend {
    pub fn from(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

pub struct GeneratedUrl {
    pub url: Url,
    pub csrf_state: CsrfToken,
    pub verifier: PkceCodeVerifier,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Default)]
pub enum Role {
    #[default]
    User,
    Admin,
}

#[derive(Error, Debug)]
pub enum UserStoreError {
    #[error("invalid id")]
    InvalidId,
    #[error("SQL error: {0:?}")]
    SqlError(<kernel::sqlite::Sqlite as kernel::domain::Storage>::Err),
}

pub trait OAuthProfile: Sized + Send + Sync + DeserializeOwned {
    const NAME: &'static str;
    const AUTH_URL: &'static str;
    const TOKEN_URL: &'static str;
    const USERINFO_URL: &'static str;

    fn auth_header(token: &str) -> String;
    fn to_user(&self) -> User;
}

// https://developers.google.com/identity/openid-connect/openid-connect#obtainuserinfo
#[derive(Deserialize, Default, Debug)]
#[allow(dead_code)]
pub struct GoogleUser {
    pub sub: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub profile: Option<String>,
    pub picture: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub gender: Option<String>,
    pub hd: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
pub struct GithubUser {
    pub login: String,
    pub id: i64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

// https://yandex.ru/dev/id/doc/dg/api-id/reference/response.html
#[derive(Deserialize, Default, Debug)]
#[allow(dead_code)]
pub struct YandexUser {
    pub login: String,
    pub id: String,
    pub real_name: Option<String>,
    pub display_name: Option<String>,
    pub default_email: Option<String>,
    pub is_avatar_empty: Option<bool>,
    pub default_avatar_id: Option<String>,
}

impl OAuthProfile for GoogleUser {
    const NAME: &'static str = "google";
    const AUTH_URL: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
    const TOKEN_URL: &'static str = "https://www.googleapis.com/oauth2/v3/token";
    const USERINFO_URL: &'static str = "https://www.googleapis.com/oauth2/v3/userinfo";

    fn auth_header(token: &str) -> String {
        format!("Bearer {token}")
    }

    fn to_user(&self) -> User {
        User {
            created: Utc::now(),
            email: self.email.clone().unwrap_or_default(),
            name: self.name.clone().unwrap_or_default(),
            login: self.email.as_deref().unwrap_or_default().to_string(),
            avatar_url: self.picture.as_deref().unwrap_or_default().to_string(),
            federated_id: self.sub.clone(),
            admin: false,
            verified: true,
            provider: Self::NAME.to_owned(),
        }
    }
}

impl OAuthProfile for GithubUser {
    const NAME: &'static str = "github";
    const AUTH_URL: &'static str = "https://github.com/login/oauth/authorize";
    const TOKEN_URL: &'static str = "https://github.com/login/oauth/access_token";
    const USERINFO_URL: &'static str = "https://api.github.com/user";

    fn auth_header(token: &str) -> String {
        format!("Bearer {token}")
    }

    fn to_user(&self) -> User {
        User {
            created: Utc::now(),
            email: self.email.as_deref().unwrap_or_default().to_string(),
            name: self.name.as_deref().unwrap_or_default().to_string(),
            login: self.login.clone(),
            avatar_url: self.avatar_url.as_deref().unwrap_or_default().to_string(),
            federated_id: format!("{}", self.id),
            admin: false,
            verified: true,
            provider: Self::NAME.to_owned(),
        }
    }
}

impl OAuthProfile for YandexUser {
    const NAME: &'static str = "yandex";
    const AUTH_URL: &'static str = "https://oauth.yandex.ru/authorize";
    const TOKEN_URL: &'static str = "https://oauth.yandex.ru/token";
    const USERINFO_URL: &'static str = "https://login.yandex.ru/info?format=json";

    fn auth_header(token: &str) -> String {
        format!("OAuth {token}")
    }

    fn to_user(&self) -> User {
        User {
            created: Utc::now(),
            email: self
                .default_email
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            name: self.display_name.as_deref().unwrap_or_default().to_string(),
            login: self.login.clone(),
            avatar_url: self
                .default_avatar_id
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            federated_id: self.id.clone(),
            admin: false,
            verified: true,
            provider: Self::NAME.to_owned(),
        }
    }
}

impl<T: OAuthProfile> OAuthAuthorizer<T> {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let (client, provider) =
            create_client_and_provider(db_path, T::NAME, T::AUTH_URL, T::TOKEN_URL)
                .with_context(|| format!("Failed to create {} authorizer", T::NAME))?;
        Ok(Self {
            client,
            provider,
            _phantom: PhantomData,
        })
    }

    #[must_use]
    pub fn generate_authorize_url(&self) -> GeneratedUrl {
        let request = self.client.authorize_url(CsrfToken::new_random).add_scopes(
            self.provider
                .scopes
                .iter()
                .map(|scope| Scope::new(scope.clone())),
        );
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = request.set_pkce_challenge(pkce_code_challenge).url();
        GeneratedUrl {
            url: authorize_url,
            csrf_state,
            verifier: pkce_code_verifier,
        }
    }

    pub async fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: PkceCodeVerifier,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let http_client = oauth2::reqwest::ClientBuilder::new()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(oauth2::reqwest::redirect::Policy::none())
            .build()?;

        let result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_code_verifier)
            .request_async(&http_client)
            .await
            .with_context(|| "Failed to exchange OAuth code with pkce verifier")?;
        Ok(result)
    }

    pub async fn get_user(&self, token: &AccessToken) -> Result<User> {
        let profile: T = send_user_request(T::USERINFO_URL, &T::auth_header(token.secret()))
            .await?
            .json()
            .await?;
        Ok(profile.to_user())
    }
}

async fn send_user_request(url: &str, auth_header: &str) -> Result<reqwest::Response> {
    let response = Client::builder()
        .build()?
        .get(url)
        .header("Authorization", auth_header)
        .header("User-Agent", "egoroff.spb.ru API auth request")
        .send()
        .await?;
    tracing::debug!("Get user status: {}", response.status());
    if response.status() == StatusCode::OK {
        Ok(response)
    } else {
        Err(anyhow::Error::msg(
            response.text().await.unwrap_or_default(),
        ))
    }
}

impl AuthUser for AppUser {
    type Id = String;

    fn id(&self) -> String {
        format!("{}_{}", self.user.provider, self.user.federated_id)
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.user.federated_id.as_bytes()
    }
}

impl AuthzBackend for AuthBackend {
    type Permission = Role;

    /// Gets the permissions for the provided user.
    async fn get_user_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let mut user_permissions = HashSet::new();
        user_permissions.insert(Role::User);
        if user.user.admin {
            user_permissions.insert(Role::Admin);
        }
        Ok(user_permissions)
    }
}

impl AuthnBackend for AuthBackend
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
{
    type User = AppUser;
    type Error = UserStoreError;
    type Credentials = AppUser;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match Sqlite::open(self.db_path.as_path(), Mode::ReadOnly) {
            Ok(storage) => {
                let user = storage.get_user(&creds.user.federated_id, &creds.user.provider);
                match user {
                    Ok(user) => Ok(Some(AppUser::new(user))),
                    Err(err) => Err(UserStoreError::SqlError(err)),
                }
            }
            Err(err) => Err(UserStoreError::SqlError(err)),
        }
    }

    async fn get_user(
        &self,
        user_id: &String,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        match Sqlite::open(self.db_path.as_path(), Mode::ReadOnly) {
            Ok(storage) => {
                let (provider, federated_id) =
                    user_id.split_once('_').ok_or(UserStoreError::InvalidId)?;
                let user = storage.get_user(federated_id, provider);
                match user {
                    Ok(user) => Ok(Some(AppUser::new(user))),
                    Err(err) => Err(UserStoreError::SqlError(err)),
                }
            }
            Err(err) => Err(UserStoreError::SqlError(err)),
        }
    }
}

fn create_client_and_provider<P: AsRef<Path>>(
    db_path: P,
    provider: &str,
    auth_uri: &str,
    token_uri: &str,
) -> Result<(SpecialClient, OAuthProvider)> {
    let storage = Sqlite::open(db_path, Mode::ReadOnly)?;

    let provider = storage.get_oauth_provider(provider)?;

    let auth = AuthUrl::new(auth_uri.to_string())?;
    let token = TokenUrl::new(token_uri.to_string())?;

    let client_id = ClientId::new(provider.client_id.clone());
    let client_secret = ClientSecret::new(provider.secret.clone());
    let client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_token_uri(token)
        .set_auth_uri(auth)
        .set_redirect_uri(RedirectUrl::new(provider.redirect_url.clone())?);
    Ok((client, provider))
}
