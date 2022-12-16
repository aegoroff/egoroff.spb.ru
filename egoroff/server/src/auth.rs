use std::path::Path;

use anyhow::Result;
use kernel::{
    domain::Storage,
    sqlite::{Mode, Sqlite},
};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl,
};

pub fn build_google_oauth_client<P: AsRef<Path>>(db_path: P) -> Result<BasicClient> {
    let storage = Sqlite::open(db_path, Mode::ReadOnly)?;

    let google = storage.get_oauth_provider("google").unwrap();

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())?;

    let google_client_id = ClientId::new(google.client_id);
    let google_client_secret = ClientSecret::new(google.secret);
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(google.redirect_url)?)
    .set_revocation_uri(RevocationUrl::new(
        "https://oauth2.googleapis.com/revoke".to_string(),
    )?);
    Ok(client)
}
