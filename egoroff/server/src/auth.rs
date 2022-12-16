use std::path::Path;

use anyhow::{Ok, Result};
use kernel::{
    domain::{OAuthProvider, Storage},
    sqlite::{Mode, Sqlite},
};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::http_client,
    url::Url,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RevocationUrl, Scope, StandardTokenResponse,
    TokenUrl,
};

pub trait Authorizer {
    fn generate_authorize_url(&self) -> (Url, CsrfToken, PkceCodeVerifier);
    fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: PkceCodeVerifier,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>;
}

pub struct GoogleAuthorizer {
    client: BasicClient,
    provider: OAuthProvider,
}

impl GoogleAuthorizer {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<GoogleAuthorizer> {
        let storage = Sqlite::open(db_path, Mode::ReadOnly)?;

        let google = storage.get_oauth_provider("google").unwrap();

        let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
        let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())?;

        let google_client_id = ClientId::new(google.client_id.clone());
        let google_client_secret = ClientSecret::new(google.secret.clone());
        let client = BasicClient::new(
            google_client_id,
            Some(google_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(google.redirect_url.clone())?)
        .set_revocation_uri(RevocationUrl::new(
            "https://oauth2.googleapis.com/revoke".to_string(),
        )?);
        Ok(Self {
            client,
            provider: google,
        })
    }
}

impl Authorizer for GoogleAuthorizer {
    fn generate_authorize_url(&self) -> (Url, CsrfToken, PkceCodeVerifier) {
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut request = self.client.authorize_url(CsrfToken::new_random);
        for scope in self.provider.scopes.iter() {
            request = request.add_scope(Scope::new(scope.clone()));
        }
        let (authorize_url, csrf_state) = request.set_pkce_challenge(pkce_code_challenge).url();
        (authorize_url, csrf_state, pkce_code_verifier)
    }

    fn exchange_code(
        &self,
        code: String,
        pkce_code_verifier: PkceCodeVerifier,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_code_verifier)
            .request(http_client)?;
        Ok(result)
    }
}
