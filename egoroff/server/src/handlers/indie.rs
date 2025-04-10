use super::*;

use axum::extract::Form;
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::{TimeDelta, Utc};

use crate::{
    body::Redirect,
    domain::PageContext,
    indie::{
        Claims, IndieQuery, ME, SCOPES, Token, TokenRequest, TokenValidationResult, generate_jwt,
        read_from_client, validate_jwt,
    },
};

pub async fn serve_auth(
    Query(query): Query<IndieQuery>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let private_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupri.pem");

    let redirect = query.redirect_uri.unwrap_or_default();
    let client_id = query.client_id.unwrap_or_default();

    if redirect.starts_with(&client_id) {
        let now = Utc::now();
        let issued = now.timestamp() as usize;
        let Some(lifetime_minutes) = TimeDelta::try_minutes(10) else {
            return bad_request_error_response(Body::empty());
        };
        let Some(expired) = now.checked_add_signed(lifetime_minutes) else {
            return bad_request_error_response(Body::empty());
        };
        let expired = expired.timestamp() as usize;
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

        let Some(state) = query.state else {
            tracing::error!("No state extracted from query");
            return bad_request_error_response(Body::empty());
        };

        // generate token and if success redirect to uri specified
        match generate_jwt(&claims, private_key_path) {
            Ok(token) => {
                let q = format!("state={state}&code={token}");
                let Some(mut to) = Resource::new(&redirect) else {
                    return bad_request_error_response(Body::empty());
                };
                let mut c = page_context.cache.lock().await;
                c.insert(token);
                to.append_query(&q);
                let to = to.to_string();
                // redirect to uri with state and new token
                let resp = Redirect::found(&to);
                (StatusCode::FOUND, resp.into_response())
            }
            Err(e) => {
                tracing::error!("generate jwt token error: {e:#?}");
                bad_request_error_response(e.to_string())
            }
        }
    } else if let Some(u) = Resource::new(&client_id) {
        match read_from_client(&u.to_string()).await {
            Ok(resp) => {
                tracing::info!("Response from client: {resp}");
                (StatusCode::OK, Body::empty().into_response())
            }
            Err(e) => {
                tracing::error!("Error reading data from client: {e:#?}");
                bad_request_error_response(Body::empty())
            }
        }
    } else {
        tracing::error!("invalid client ID: {client_id}");
        bad_request_error_response(Body::empty())
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
) -> impl IntoResponse {
    let public_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupub.pem");

    match validate_jwt(&req.code, public_key_path) {
        Ok(_claims) => {
            let mut cache = page_context.cache.lock().await;
            cache.remove(&req.code);
        }
        Err(e) => {
            tracing::error!("validate jwt token error: {e:#?}");
            return unauthorized_response(e.to_string());
        }
    }

    let client_id = req.client_id;
    let redirect_uri = req.redirect_uri;
    let now = Utc::now();
    let issued = now.timestamp() as usize;
    let claims = Claims {
        client_id,
        redirect_uri: Some(redirect_uri),
        aud: None,
        exp: None,
        iat: Some(issued),
        iss: Some(ME.to_string()),
        nbf: None,
        sub: None,
        jti: None,
    };

    let private_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupri.pem");
    match generate_jwt(&claims, private_key_path) {
        Ok(token) => {
            let t = Token {
                access_token: token,
                token_type: "Bearer".to_string(),
                scope: SCOPES.to_string(),
                me: ME.to_string(),
            };
            success_response(Json(t))
        }
        Err(e) => {
            tracing::error!("generate jwt token error: {e:#?}");
            bad_request_error_response(e.to_string())
        }
    }
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
) -> impl IntoResponse {
    let public_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupub.pem");

    match validate_jwt(authorization.token(), public_key_path) {
        Ok(claims) => {
            let Some(me) = claims.iss else {
                return unauthorized_response("no iss".to_string());
            };

            let response = TokenValidationResult {
                me,
                client_id: claims.client_id,
                scope: SCOPES.to_string(),
            };
            success_response(Json(response))
        }
        Err(e) => {
            tracing::error!("validate jwt token error: {e:#?}");
            unauthorized_response(e.to_string())
        }
    }
}
