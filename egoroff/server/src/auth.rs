use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use axum_login::{secrecy::SecretVec, AuthUser, UserStore};
use chrono::Utc;
use kernel::{
    domain::{OAuthProvider, Storage, User},
    sqlite::{Mode, Sqlite},
};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    url::Url,
    AccessToken, AuthUrl, AuthorizationCode, AuthorizationRequest, ClientId, ClientSecret,
    CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    StandardTokenResponse, TokenUrl,
};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

#[derive(Clone)]
pub struct UserStorage {
    db_path: Arc<PathBuf>,
}

impl UserStorage {
    pub fn from(db_path: Arc<PathBuf>) -> Self {
        Self { db_path }
    }
}

pub struct GeneratedUrl {
    pub url: Url,
    pub csrf_state: CsrfToken,
    pub verifier: Option<PkceCodeVerifier>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Role {
    User,
    Admin,
}

pub trait ToUser {
    fn to_user(&self) -> User;
}

// https://developers.google.com/identity/openid-connect/openid-connect#obtainuserinfo
#[derive(Deserialize, Default, Debug)]
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
            login: self.email.clone().unwrap_or_default(),
            avatar_url: self.picture.clone().unwrap_or_default(),
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
            email: self.email.clone().unwrap_or_default(),
            name: self.name.clone().unwrap_or_default(),
            login: self.login.clone(),
            avatar_url: self.avatar_url.clone().unwrap_or_default(),
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
            email: self.default_email.clone().unwrap_or_default(),
            name: self.display_name.clone().unwrap_or_default(),
            login: self.login.clone(),
            avatar_url: self.default_avatar_id.clone().unwrap_or_default(),
            federated_id: self.id.clone(),
            admin: false,
            verified: true,
            provider: "yandex".to_owned(),
        }
    }
}

#[async_trait]
pub trait Authorizer<T> {
    fn generate_authorize_url(&self) -> GeneratedUrl;
    async fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: Option<PkceCodeVerifier>,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>;
    async fn get_user(&self, token: &AccessToken) -> Result<T>;
}

pub struct GoogleAuthorizer {
    client: BasicClient,
    provider: OAuthProvider,
}

pub struct GithubAuthorizer {
    client: BasicClient,
    provider: OAuthProvider,
}

pub struct YandexAuthorizer {
    client: BasicClient,
    provider: OAuthProvider,
}

fn make_auth_request<'a>(
    client: &'a BasicClient,
    provider: &OAuthProvider,
) -> AuthorizationRequest<'a> {
    let request = client.authorize_url(CsrfToken::new_random);
    request.add_scopes(
        provider
            .scopes
            .iter()
            .map(|scope| Scope::new(scope.clone())),
    )
}

async fn exchange_code(
    client: &BasicClient,
    code: String,
    pkce_code_verifier: Option<PkceCodeVerifier>,
) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
    let result = match pkce_code_verifier {
        Some(verifier) => {
            client
                .exchange_code(AuthorizationCode::new(code))
                .set_pkce_verifier(verifier)
                .request_async(async_http_client)
                .await?
        }
        None => {
            client
                .exchange_code(AuthorizationCode::new(code))
                .request_async(async_http_client)
                .await?
        }
    };
    Ok(result)
}

impl GoogleAuthorizer {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<GoogleAuthorizer> {
        let (client, provider) = create_client_and_provider(
            db_path,
            "google",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://www.googleapis.com/oauth2/v3/token",
        )?;
        Ok(Self { client, provider })
    }
}

impl GithubAuthorizer {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<GithubAuthorizer> {
        let (client, provider) = create_client_and_provider(
            db_path,
            "github",
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        )?;
        Ok(Self { client, provider })
    }
}

impl YandexAuthorizer {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<YandexAuthorizer> {
        let (client, provider) = create_client_and_provider(
            db_path,
            "yandex",
            "https://oauth.yandex.ru/authorize",
            "https://oauth.yandex.ru/token",
        )?;
        Ok(Self { client, provider })
    }
}

#[async_trait]
impl Authorizer<GoogleUser> for GoogleAuthorizer {
    fn generate_authorize_url(&self) -> GeneratedUrl {
        let request = make_auth_request(&self.client, &self.provider);
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = request.set_pkce_challenge(pkce_code_challenge).url();
        GeneratedUrl {
            url: authorize_url,
            csrf_state,
            verifier: Some(pkce_code_verifier),
        }
    }

    async fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: Option<PkceCodeVerifier>,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        exchange_code(&self.client, code, pkce_code_verifier).await
    }

    async fn get_user(&self, token: &AccessToken) -> Result<GoogleUser> {
        let uri = "https://www.googleapis.com/oauth2/v3/userinfo";

        let auth_value = format!("Bearer {}", token.secret());

        let client = Client::builder().build()?;

        let response = client
            .get(uri)
            .header("Authorization", auth_value)
            .send()
            .await?;
        tracing::debug!("Get user status: {}", response.status());
        let user = response.json::<GoogleUser>().await?;
        Ok(user)
    }
}

#[async_trait]
impl Authorizer<GithubUser> for GithubAuthorizer {
    fn generate_authorize_url(&self) -> GeneratedUrl {
        let request = make_auth_request(&self.client, &self.provider);
        let (authorize_url, csrf_state) = request.url();
        GeneratedUrl {
            url: authorize_url,
            csrf_state,
            verifier: None,
        }
    }

    async fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: Option<PkceCodeVerifier>,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        exchange_code(&self.client, code, pkce_code_verifier).await
    }

    async fn get_user(&self, token: &AccessToken) -> Result<GithubUser> {
        let uri = "https://api.github.com/user";

        let auth_value = format!("Bearer {}", token.secret());

        let client = Client::builder().build()?;

        let response = client
            .get(uri)
            .header("Authorization", auth_value)
            .header("User-Agent", "egoroff.spb.ru API auth request")
            .send()
            .await?;
        tracing::debug!("Get user status: {}", response.status());
        if response.status() == StatusCode::OK {
            let user = response.json::<GithubUser>().await?;
            Ok(user)
        } else {
            let error = response.text().await.unwrap_or_default();
            let err = anyhow::Error::msg(error);
            Err(err)
        }
    }
}

#[async_trait]
impl Authorizer<YandexUser> for YandexAuthorizer {
    fn generate_authorize_url(&self) -> GeneratedUrl {
        let request = make_auth_request(&self.client, &self.provider);
        let (authorize_url, csrf_state) = request.url();
        GeneratedUrl {
            url: authorize_url,
            csrf_state,
            verifier: None,
        }
    }

    async fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: Option<PkceCodeVerifier>,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        exchange_code(&self.client, code, pkce_code_verifier).await
    }

    async fn get_user(&self, token: &AccessToken) -> Result<YandexUser> {
        let uri = "https://login.yandex.ru/info?format=json";

        let auth_value = format!("OAuth {}", token.secret());

        let client = Client::builder().build()?;

        let response = client
            .get(uri)
            .header("Authorization", auth_value)
            .header("User-Agent", "egoroff.spb.ru API auth request")
            .send()
            .await?;
        tracing::debug!("Get user status: {}", response.status());
        if response.status() == StatusCode::OK {
            let user = response.json::<YandexUser>().await?;
            Ok(user)
        } else {
            let error = response.text().await.unwrap_or_default();
            let err = anyhow::Error::msg(error);
            Err(err)
        }
    }
}

impl AuthUser<Role> for User {
    fn get_id(&self) -> String {
        format!("{}_{}", self.provider, self.federated_id)
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.federated_id.clone().into())
    }

    fn get_role(&self) -> Option<Role> {
        if self.admin {
            Some(Role::Admin)
        } else {
            Some(Role::User)
        }
    }
}

#[async_trait]
impl UserStore<Role> for UserStorage
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
{
    type User = User;

    async fn load_user(
        &self,
        user_id: &str,
    ) -> std::result::Result<Option<Self::User>, eyre::Error> {
        let storage = Sqlite::open(self.db_path.as_path(), Mode::ReadOnly)?;
        let mut id_parts = user_id.split('_');
        let provider = id_parts
            .next()
            .ok_or_else(|| eyre::Error::msg("invalid id"))?;
        let federated_id = id_parts
            .next()
            .ok_or_else(|| eyre::Error::msg("invalid id"))?;
        let user = storage.get_user(federated_id, provider);
        match user {
            Ok(user) => Ok(Some(user)),
            Err(err) => {
                let msg = format!("{err}");
                Err(eyre::Error::msg(msg))
            }
        }
    }
}

fn create_client_and_provider<P: AsRef<Path>>(
    db_path: P,
    provider: &str,
    auth_uri: &str,
    token_uri: &str,
) -> Result<(BasicClient, OAuthProvider)> {
    let storage = Sqlite::open(db_path, Mode::ReadOnly)?;

    let provider = storage.get_oauth_provider(provider)?;

    let auth_url = AuthUrl::new(auth_uri.to_string())?;
    let token_url = TokenUrl::new(token_uri.to_string())?;

    let client_id = ClientId::new(provider.client_id.clone());
    let client_secret = ClientSecret::new(provider.secret.clone());
    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(RedirectUrl::new(provider.redirect_url.clone())?);
    Ok((client, provider))
}
