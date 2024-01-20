#![allow(clippy::module_name_repetitions)]

use std::{collections::HashSet, fs, marker::PhantomData, path::Path, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    body::HttpBody,
    http::{self, Request, Response},
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower_http::validate_request::ValidateRequest;
use utoipa::ToSchema;

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

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub me: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub me: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TokenValidationResult {
    pub me: String,
    pub client_id: String,
    pub scope: String,
}

#[derive(Debug, Error)]
pub enum IndieAuthError {
    #[error("No authorization header extracted from request")]
    MissingAuthorizationHeader,
    #[error("No authorization header value extracted from request")]
    MissingAuthorizationHeaderValue,
    #[error("Authorization header not started from Bearer")]
    NotStarterFromBearer,
}

pub fn generate_jwt<P: AsRef<Path>>(claims: &Claims, private_key_path: P) -> Result<String> {
    let data = fs::read(private_key_path)
        .with_context(|| "Private key cannot be read using path specified")?;
    let token = encode(
        &Header::new(Algorithm::RS256),
        claims,
        &EncodingKey::from_rsa_pem(&data)?,
    )?;
    Ok(token)
}

pub fn validate_jwt<P: AsRef<Path>>(token: &str, public_key_path: P) -> Result<Claims> {
    let data = fs::read(public_key_path)
        .with_context(|| "Public key cannot be read using path specified")?;
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

impl<Req, Resp> ValidateRequest<Req> for Indie<Resp>
where
    Resp: HttpBody + Default,
{
    type ResponseBody = Resp;

    fn validate(&mut self, request: &mut Request<Req>) -> Result<(), Response<Self::ResponseBody>> {
        let unauthorized_response = Response::builder()
            .status(http::StatusCode::UNAUTHORIZED)
            .body(Default::default())
            .unwrap_or_default();

        let Some(value) = request.headers().get("authorization") else {
            tracing::error!("{}", IndieAuthError::MissingAuthorizationHeader.to_string());
            return Err(unauthorized_response);
        };
        let Ok(auth_header) = value.to_str() else {
            tracing::error!(
                "{}",
                IndieAuthError::MissingAuthorizationHeaderValue.to_string()
            );
            return Err(unauthorized_response);
        };

        let Some(token) = auth_header.strip_prefix("Bearer ") else {
            tracing::error!("{}", IndieAuthError::NotStarterFromBearer.to_string());
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

/// A wrapper around [`tower_http::validate_request::ValidateRequestHeaderLayer`] which
/// provides login authorization.
pub struct RequireIndieAuthorizationLayer;

impl RequireIndieAuthorizationLayer {
    /// Authorizes requests by requiring valid Indie auth token in authorization header, otherwise it rejects
    /// with [`http::StatusCode::UNAUTHORIZED`].
    pub fn auth<Resp: HttpBody + Default>(
        public_key_path: Arc<String>,
    ) -> tower_http::validate_request::ValidateRequestHeaderLayer<Indie<Resp>> {
        tower_http::validate_request::ValidateRequestHeaderLayer::custom(Indie::<_> {
            public_key_path,
            _body_type: PhantomData,
        })
    }
}
