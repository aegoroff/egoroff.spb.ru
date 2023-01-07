use std::{fs, path::Path};

use anyhow::Result;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

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

pub fn generate_jwt<P: AsRef<Path>>(claims: Claims, private_key_path: P) -> Result<String> {
    let data = fs::read(private_key_path)?;
    let token = encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(&data)?,
    )?;
    Ok(token)
}
