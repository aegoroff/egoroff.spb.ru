use std::{collections::HashSet, fs, marker::PhantomData, path::Path, sync::Arc};

use anyhow::Result;
use axum::{
    body::HttpBody,
    http::{self, Request, Response},
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tower_http::auth::AuthorizeRequest;

pub const ME: &str = "https://www.egoroff.spb.ru/";
pub const SCOPES: &str = "create media delete";

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub client_id: String,
    pub redirect_uri: Option<String>,
    pub aud: Option<String>, // Optional. Audience
    pub exp: Option<usize>, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
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
    let mut required_claims = HashSet::new();
    required_claims.insert("iss".to_string());
    required_claims.insert("iat".to_string());
    required_claims.insert("redirect_uri".to_string());
    required_claims.insert("client_id".to_string());
    validation.required_spec_claims = required_claims;
    let claims = decode::<Claims>(token, &DecodingKey::from_rsa_pem(&data)?, &validation)?;

    Ok(claims.claims)
}

pub async fn read_from_client(uri: &str) -> Result<String> {
    let client = Client::builder().build()?;

    let response = client.get(uri).send().await?.text().await?;

    Ok(response)
}

pub struct Indie<ResBody> {
    public_key_path: Arc<String>,
    _body_type: PhantomData<fn() -> ResBody>,
}

impl<ResBody> Clone for Indie<ResBody> {
    fn clone(&self) -> Self {
        Self {
            public_key_path: self.public_key_path.clone(),
            _body_type: PhantomData,
        }
    }
}

impl<ReqBody, ResBody> AuthorizeRequest<ReqBody> for Indie<ResBody>
where
    ResBody: HttpBody + Default,
{
    type ResponseBody = ResBody;

    fn authorize(
        &mut self,
        request: &mut Request<ReqBody>,
    ) -> Result<(), Response<Self::ResponseBody>> {
        let unauthorized_response = Response::builder()
            .status(http::StatusCode::UNAUTHORIZED)
            .body(Default::default())
            .unwrap();

        let value = if let Some(h) = request.headers().get("authorization") {
            h
        } else {
            tracing::error!("No authorization header extracted from request");
            return Err(unauthorized_response);
        };
        let auth_header = if let Ok(val) = value.to_str() {
            val
        } else {
            tracing::error!("No authorization header value extracted from request");
            return Err(unauthorized_response);
        };

        let token = if let Some(val) = auth_header.strip_prefix("Bearer ") {
            val
        } else {
            tracing::error!("Authorization header not started from Bearer");
            return Err(unauthorized_response);
        };

        match validate_jwt(token, self.public_key_path.as_str()) {
            Ok(_claims) => Ok(()),
            Err(e) => {
                tracing::error!("Token {token} validation error: {e:#?}");
                Err(unauthorized_response)
            }
        }
    }
}

/// A wrapper around [`tower_http::auth::RequireAuthorizationLayer`] which
/// provides login authorization.
pub struct RequireIndieAuthorizationLayer;

impl RequireIndieAuthorizationLayer {
    /// Authorizes requests by requiring valid Indie auth token in authorization header, otherwise it rejects
    /// with [`http::StatusCode::UNAUTHORIZED`].
    pub fn auth<ResBody>(
        public_key_path: Arc<String>,
    ) -> tower_http::auth::RequireAuthorizationLayer<Indie<ResBody>>
    where
        ResBody: HttpBody + Default,
    {
        tower_http::auth::RequireAuthorizationLayer::custom(Indie::<_> {
            public_key_path,
            _body_type: PhantomData,
        })
    }
}