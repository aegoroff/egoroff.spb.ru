#![allow(clippy::module_name_repetitions)]

use std::marker::PhantomData;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use async_trait::async_trait;
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
use serde::Deserialize;
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

// Single struct instead of three separate ones
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

pub trait ToUser {
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

impl ToUser for GoogleUser {
    fn to_user(&self) -> User {
        let created = Utc::now();
        User {
            created,
            email: self.email.clone().unwrap_or_default(),
            name: self.name.clone().unwrap_or_default(),
            login: self.email.as_deref().unwrap_or_default().to_string(),
            avatar_url: self.picture.as_deref().unwrap_or_default().to_string(),
            federated_id: self.sub.clone(),
            admin: false,
            verified: true,
            provider: "google".to_owned(),
        }
    }
}

impl ToUser for GithubUser {
    fn to_user(&self) -> User {
        let created = Utc::now();
        User {
            created,
            email: self.email.as_deref().unwrap_or_default().to_string(),
            name: self.name.as_deref().unwrap_or_default().to_string(),
            login: self.login.clone(),
            avatar_url: self.avatar_url.as_deref().unwrap_or_default().to_string(),
            federated_id: format!("{}", self.id),
            admin: false,
            verified: true,
            provider: "github".to_owned(),
        }
    }
}

impl ToUser for YandexUser {
    fn to_user(&self) -> User {
        let created = Utc::now();
        User {
            created,
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
            provider: "yandex".to_owned(),
        }
    }
}

#[async_trait]
pub trait FetchUser: Sized + Send + Sync {
    async fn fetch(token: &AccessToken) -> Result<Self>;
}

#[async_trait]
impl FetchUser for GoogleUser {
    async fn fetch(token: &AccessToken) -> Result<Self> {
        Ok(send_user_request(
            "https://www.googleapis.com/oauth2/v3/userinfo",
            &format!("Bearer {}", token.secret()),
        )
        .await?
        .json()
        .await?)
    }
}

#[async_trait]
impl FetchUser for GithubUser {
    async fn fetch(token: &AccessToken) -> Result<Self> {
        Ok(send_user_request(
            "https://api.github.com/user",
            &format!("Bearer {}", token.secret()),
        )
        .await?
        .json()
        .await?)
    }
}

#[async_trait]
impl FetchUser for YandexUser {
    async fn fetch(token: &AccessToken) -> Result<Self> {
        Ok(send_user_request(
            "https://login.yandex.ru/info?format=json",
            &format!("OAuth {}", token.secret()),
        )
        .await?
        .json()
        .await?)
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

#[async_trait]
pub trait Authorizer<T> {
    fn generate_authorize_url(&self) -> GeneratedUrl;
    async fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: PkceCodeVerifier,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>;
    async fn get_user(&self, token: &AccessToken) -> Result<T>;
}

impl OAuthAuthorizer<GoogleUser> {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let (client, provider) = create_client_and_provider(
            db_path,
            "google",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://www.googleapis.com/oauth2/v3/token",
        )
        .with_context(|| "Failed to create Google authorizer")?;
        Ok(Self {
            client,
            provider,
            _phantom: PhantomData,
        })
    }
}

impl OAuthAuthorizer<GithubUser> {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let (client, provider) = create_client_and_provider(
            db_path,
            "github",
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        )
        .with_context(|| "Failed to create GitHub authorizer")?;
        Ok(Self {
            client,
            provider,
            _phantom: PhantomData,
        })
    }
}

impl OAuthAuthorizer<YandexUser> {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let (client, provider) = create_client_and_provider(
            db_path,
            "yandex",
            "https://oauth.yandex.ru/authorize",
            "https://oauth.yandex.ru/token",
        )
        .with_context(|| "Failed to create Yandex authorizer")?;
        Ok(Self {
            client,
            provider,
            _phantom: PhantomData,
        })
    }
}

#[async_trait]
impl<T: FetchUser> Authorizer<T> for OAuthAuthorizer<T> {
    fn generate_authorize_url(&self) -> GeneratedUrl {
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

    async fn exchange_code(
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

    async fn get_user(&self, token: &AccessToken) -> Result<T> {
        T::fetch(token).await
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
