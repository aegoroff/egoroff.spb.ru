use std::{fs, path::Path};

use anyhow::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub const ME: &str = "https://www.egoroff.spb.ru/";
pub const SCOPES: &str = "create media delete";

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub client_id: String,
    pub redirect_uri: String,
    pub aud: Option<String>, // Optional. Audience
    pub exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: Option<usize>, // Optional. Issued at (as UTC timestamp)
    pub iss: Option<String>, // Optional. Issuer
    pub nbf: Option<usize>, // Optional. Not Before (as UTC timestamp)
    pub sub: Option<String>, // Optional. Subject (whom token refers to)
    pub jti: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct IndieQuery {
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub me: String,
}

#[derive(Deserialize, Serialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub me: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenValidationResult {
    pub me: String,
    pub client_id: String,
    pub scope: String,
}

pub fn generate_jwt<P: AsRef<Path>>(claims: Claims, private_key_path: P) -> Result<String> {
    let data = fs::read(private_key_path)?;
    let token = encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(&data)?,
    )?;
    Ok(token)
}

pub fn validate_jwt<P: AsRef<Path>>(token: &str, public_key_path: P) -> Result<Claims> {
    let data = fs::read(public_key_path)?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ME]);
    let claims = decode::<Claims>(token, &DecodingKey::from_rsa_pem(&data)?, &validation)?;

    Ok(claims.claims)
}

pub async fn read_from_client(uri: &str) -> Result<String> {
    let client = Client::builder().build()?;

    let response = client.get(uri).send().await?.text().await?;

    Ok(response)
}
