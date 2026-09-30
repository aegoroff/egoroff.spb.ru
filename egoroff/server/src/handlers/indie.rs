use super::*;

use axum::extract::Form;
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::{TimeDelta, Utc};

use crate::{
    domain::PageContext,
    indie::{
        Claims, IndieQuery, ME, SCOPES, Token, TokenRequest, TokenValidationResult, generate_jwt,
        read_from_client, validate_jwt,
    },
};
use axum::http::header::LOCATION;

pub async fn serve_auth(
    Query(query): Query<IndieQuery>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> Result<Response, ApiError> {
    let private_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupri.pem");

    let redirect = query.redirect_uri.unwrap_or_default();
    let client_id = query.client_id.unwrap_or_default();

    if redirect.starts_with(&client_id) {
        let now = Utc::now();
        let issued = now.timestamp() as usize;
        let expired = TimeDelta::try_minutes(10)
            .and_then(|lifetime| now.checked_add_signed(lifetime))
            .ok_or_else(|| ApiError::internal("invalid Indie code lifetime"))?
            .timestamp() as usize;
        let claims = Claims {
            client_id,
            redirect_uri: Some(redirect.clone()),
            aud: None,
            exp: Some(expired),
            iat: Some(issued),
            iss: Some(ME.to_string()),
            nbf: None,
            sub: None,
            jti: None,
        };

        let state = query
            .state
            .ok_or_else(|| ApiError::bad_request("state is required").logged())?;
        let mut to = Resource::new(&redirect)
            .ok_or_else(|| ApiError::bad_request("invalid redirect_uri"))?;
        let token = generate_jwt(&claims, private_key_path)?;

        to.append_query(&format!("state={state}&code={token}"));
        page_context.cache.lock().await.insert(token);
        Ok((StatusCode::FOUND, [(LOCATION, to.to_string())]).into_response())
    } else {
        let client = Resource::new(&client_id).ok_or_else(|| {
            ApiError::bad_request(format!("invalid client_id: {client_id}")).logged()
        })?;
        let resp = read_from_client(&client.to_string()).await.map_err(|e| {
            ApiError::bad_request("cannot read client_id")
                .caused_by(e.context("Error reading data from client"))
        })?;
        tracing::info!("Response from client: {resp}");
        Ok(StatusCode::OK.into_response())
    }
}

/// Generates Indie authorization JWT token
#[utoipa::path(
    post,
    path = "/token",
    request_body(content = TokenRequest, content_type = "application/x-www-form-urlencoded"),
    responses(
        (status = 200, description = "Configuration read successfully", body = Token),
        (status = 401, description = "Claims validation failed", body = String),
    ),
    tag = "indie",
)]
pub async fn serve_token_generate(
    State(page_context): State<Arc<PageContext<'_>>>,
    Form(req): Form<TokenRequest>,
) -> Result<Json<Token>, ApiError> {
    let public_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupub.pem");
    validate_jwt(&req.code, public_key_path).map_err(jwt_rejected)?;
    page_context.cache.lock().await.remove(&req.code);

    let now = Utc::now();
    let issued = now.timestamp() as usize;
    let lifetime = TimeDelta::try_days(90)
        .ok_or_else(|| ApiError::internal("invalid Indie token lifetime"))?;
    let expired = now
        .checked_add_signed(lifetime)
        .map(|dt| dt.timestamp() as usize);

    let claims = Claims {
        client_id: req.client_id,
        redirect_uri: Some(req.redirect_uri),
        aud: None,
        exp: expired,
        iat: Some(issued),
        iss: Some(ME.to_string()),
        nbf: None,
        sub: None,
        jti: None,
    };

    let private_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupri.pem");
    let access_token = generate_jwt(&claims, private_key_path)?;
    Ok(Json(Token {
        access_token,
        token_type: "Bearer".to_string(),
        scope: SCOPES.to_string(),
        me: ME.to_string(),
    }))
}

/// Validates Indie authorization JWT token that passed in Authorization header
#[utoipa::path(
    get,
    path = "/token",
    responses(
        (status = 200, description = "Configuration read successfully", body = TokenValidationResult),
        (status = 401, description = "Token validation failed"),
    ),
    tag = "indie",
    security(
        (),
        ("authorization" = [])
    )
)]
pub async fn serve_token_validate(
    State(page_context): State<Arc<PageContext<'_>>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<TokenValidationResult>, ApiError> {
    let public_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupub.pem");
    let claims = validate_jwt(authorization.token(), public_key_path).map_err(jwt_rejected)?;
    let me = claims.iss.ok_or_else(|| ApiError::unauthorized("no iss"))?;

    Ok(Json(TokenValidationResult {
        me,
        client_id: claims.client_id,
        scope: SCOPES.to_string(),
    }))
}

fn jwt_rejected(e: anyhow::Error) -> ApiError {
    ApiError::unauthorized(e.to_string()).caused_by(e.context("JWT validation failed"))
}
