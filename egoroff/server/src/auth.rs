use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use axum_login::{secrecy::SecretVec, AuthUser, UserStore};
use kernel::{
    domain::{OAuthProvider, Storage, User},
    sqlite::{Mode, Sqlite},
};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    url::Url,
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
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
}

#[async_trait]
pub trait Authorizer {
    fn generate_authorize_url(&self) -> GeneratedUrl;
    async fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: Option<PkceCodeVerifier>,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>;
}

pub struct GoogleAuthorizer {
    client: BasicClient,
    provider: OAuthProvider,
}

pub struct GithubAuthorizer {
    client: BasicClient,
    provider: OAuthProvider,
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

    pub async fn get_user(token: &AccessToken) -> Result<GoogleUser> {
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

    pub async fn get_user(token: &AccessToken) -> Result<GithubUser> {
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
        if response.status() != StatusCode::OK {
            let error = response.text().await.unwrap_or_default();
            let err = anyhow::Error::msg(error);
            Err(err)
        } else {
            let user = response.json::<GithubUser>().await?;
            Ok(user)
        }
    }
}

#[async_trait]
impl Authorizer for GoogleAuthorizer {
    fn generate_authorize_url(&self) -> GeneratedUrl {
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut request = self.client.authorize_url(CsrfToken::new_random);
        for scope in self.provider.scopes.iter() {
            request = request.add_scope(Scope::new(scope.clone()));
        }
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
        let result = match pkce_code_verifier {
            Some(verifier) => {
                self.client
                    .exchange_code(AuthorizationCode::new(code))
                    .set_pkce_verifier(verifier)
                    .request_async(async_http_client)
                    .await?
            }
            None => {
                self.client
                    .exchange_code(AuthorizationCode::new(code))
                    .request_async(async_http_client)
                    .await?
            }
        };
        Ok(result)
    }
}

#[async_trait]
impl Authorizer for GithubAuthorizer {
    fn generate_authorize_url(&self) -> GeneratedUrl {
        let mut request = self.client.authorize_url(CsrfToken::new_random);
        for scope in self.provider.scopes.iter() {
            request = request.add_scope(Scope::new(scope.clone()));
        }
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
        _pkce_code_verifier: Option<PkceCodeVerifier>,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await?;
        Ok(result)
    }
}

impl AuthUser<Role> for User {
    fn get_id(&self) -> String {
        self.federated_id.to_owned()
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
        let user = storage.get_federated_user(user_id);
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

    let provider = storage.get_oauth_provider(provider).unwrap();

    let auth_url = AuthUrl::new(auth_uri.to_string())?;
    let token_url = TokenUrl::new(token_uri.to_string())?;

    let client_id = ClientId::new(provider.client_id.clone());
    let client_secret = ClientSecret::new(provider.secret.clone());
    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(RedirectUrl::new(provider.redirect_url.clone())?);
    Ok((client, provider))
}
