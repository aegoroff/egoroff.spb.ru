use std::path::Path;

use anyhow::{Ok, Result};
use kernel::{
    domain::{OAuthProvider, Storage},
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
use reqwest::Client;
use serde::Deserialize;

pub struct GeneratedUrl {
    pub url: Url,
    pub csrf_state: CsrfToken,
    pub verifier: Option<PkceCodeVerifier>,
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
            .send()
            .await?;
        tracing::debug!("Get user status: {}", response.status());
        let user = response.json::<GithubUser>().await?;
        Ok(user)
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
