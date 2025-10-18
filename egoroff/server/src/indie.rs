#![allow(clippy::module_name_repetitions)]

use std::{collections::HashSet, fs, marker::PhantomData, path::Path, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    body::HttpBody,
    http::{self, Request, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower_http::validate_request::ValidateRequest;
use utoipa::ToSchema;

pub const ME: &str = "https://www.egoroff.spb.ru/";
pub const SCOPES: &str = "create media delete";

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize, Clone)]
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

/// Query parameters received in the IndieAuth authorization request.
/// These are extracted from the incoming query string.
#[derive(Deserialize, Serialize)]
pub struct IndieQuery {
    /// Optional client identifier supplied by the client application.
    pub client_id: Option<String>,
    /// Optional redirect URI to which the response should be sent.
    pub redirect_uri: Option<String>,
    /// Optional state value to maintain state between request and callback.
    pub state: Option<String>,
}

/// Request payload sent by the client to exchange an authorization code for an access token.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct TokenRequest {
    /// The type of grant requested; for IndieAuth this is typically `"authorization_code"`.
    pub grant_type: String,
    /// The authorization code received from the IndieAuth provider.
    pub code: String,
    /// The client identifier of the application making the request.
    pub client_id: String,
    /// The redirect URI that matches the one used during authorization.
    pub redirect_uri: String,
    /// The resource owner's identifier (the “me” URL).
    pub me: String,
}

/// Access token response returned to the client.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct Token {
    /// The issued access token string.
    pub access_token: String,
    /// The type of the token; for IndieAuth this is typically `"Bearer"`.
    pub token_type: String,
    /// Scopes granted to the access token.
    pub scope: String,
    /// The resource owner's identifier (the “me” URL).
    pub me: String,
}

/// Result of validating an access token against the IndieAuth provider.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct TokenValidationResult {
    /// The resource owner's identifier (the “me” URL).
    pub me: String,
    /// The client identifier that was authorized to obtain the token.
    pub client_id: String,
    /// Scopes that were validated for the token.
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
