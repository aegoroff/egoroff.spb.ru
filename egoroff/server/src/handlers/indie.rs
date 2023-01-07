use super::*;

use axum::{
    extract::Form,
    http::HeaderMap,
};
use chrono::{Duration, Months, Utc};

use crate::{
    body::Redirect,
    domain::PageContext,
    indie::{
        generate_jwt, read_from_client, validate_jwt, Claims, IndieQuery, Token, TokenRequest,
        TokenValidationResult, ME, SCOPES,
    },
};

pub async fn serve_auth(
    Query(query): Query<IndieQuery>,
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let private_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupri.pem");

    let redirect = query.redirect_uri.unwrap_or_default();
    let client_id = query.client_id.unwrap_or_default();

    if !redirect.is_empty() && redirect.starts_with(&client_id) {
        let now = Utc::now();
        let issued = now.timestamp() as usize;
        let expired = now.checked_add_signed(Duration::minutes(10)).unwrap();
        let expired = expired.timestamp() as usize;
        let claims = Claims {
            client_id,
            redirect_uri: redirect.clone(),
            aud: None,
            exp: expired,
            iat: Some(issued),
            iss: Some(ME.to_string()),
            nbf: None,
            sub: None,
            jti: None,
        };

        let state = if let Some(state) = query.state {
            state
        } else {
            tracing::error!("No state extracted from query");
            return (StatusCode::BAD_REQUEST, Empty::new().into_response());
        };

        match generate_jwt(claims, private_key_path) {
            Ok(token) => {
                let q = format!("state={state}&code={token}");
                let mut c = page_context.cache.lock().await;
                c.insert(token);
                let mut resource = Resource::new(&redirect).unwrap();
                resource.append_query(&q);
                let to = resource.to_string();
                let resp = Redirect::found(&to);
                (StatusCode::FOUND, resp.into_response())
            }
            Err(e) => {
                tracing::error!("generate jwt token error: {e:#?}");
                (StatusCode::BAD_REQUEST, e.to_string().into_response())
            }
        }
    } else if let Some(u) = Resource::new(&client_id) {
        match read_from_client(&u.to_string()).await {
            Ok(resp) => {
                tracing::info!("Response from client: {resp}");
                (StatusCode::OK, Empty::new().into_response())
            }
            Err(e) => {
                tracing::error!("Error reading data from client: {e:#?}");
                (StatusCode::BAD_REQUEST, Empty::new().into_response())
            }
        }
    } else {
        tracing::error!("invalid client ID: {client_id}");
        (StatusCode::BAD_REQUEST, Empty::new().into_response())
    }
}

pub async fn serve_token_generate(
    Extension(page_context): Extension<Arc<PageContext>>,
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
            return (StatusCode::BAD_REQUEST, e.to_string().into_response());
        }
    }

    let client_id = req.client_id;
    let redirect_uri = req.redirect_uri;
    let now = Utc::now();
    let issued = now.timestamp() as usize;
    let expired = now.checked_add_months(Months::new(12)).unwrap();
    let expired = expired.timestamp() as usize;
    let claims = Claims {
        client_id,
        redirect_uri,
        aud: None,
        exp: expired,
        iat: Some(issued),
        iss: Some(ME.to_string()),
        nbf: None,
        sub: None,
        jti: None,
    };

    let private_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupri.pem");
    match generate_jwt(claims, private_key_path) {
        Ok(token) => {
            let t = Token {
                access_token: token,
                token_type: "Bearer".to_string(),
                scope: SCOPES.to_string(),
                me: ME.to_string(),
            };
            (StatusCode::OK, Json(t).into_response())
        }
        Err(e) => {
            tracing::error!("generate jwt token error: {e:#?}");
            (StatusCode::BAD_REQUEST, e.to_string().into_response())
        }
    }
}

pub async fn serve_token_validate(
    Extension(page_context): Extension<Arc<PageContext>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let public_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupub.pem");

    let value = if let Some(h) = headers.get("authorization") {
        h
    } else {
        tracing::error!("No authorization header extracted from request");
        return (StatusCode::BAD_REQUEST, Empty::new().into_response());
    };
    let auth_header = if let Ok(val) = value.to_str() {
        val
    } else {
        tracing::error!("No authorization header value extracted from request");
        return (StatusCode::BAD_REQUEST, Empty::new().into_response());
    };

    let token = if let Some(val) = auth_header.strip_prefix("Bearer ") {
        val
    } else {
        tracing::error!("Authorization header not started from Bearer");
        return (StatusCode::BAD_REQUEST, Empty::new().into_response());
    };
    match validate_jwt(token, public_key_path) {
        Ok(claims) => {
            let response = TokenValidationResult {
                me: claims.iss.unwrap(),
                client_id: claims.client_id,
                scope: SCOPES.to_string(),
            };
            (StatusCode::OK, Json(response).into_response())
        }
        Err(e) => {
            tracing::error!("validate jwt token error: {e:#?}");
            (StatusCode::BAD_REQUEST, e.to_string().into_response())
        }
    }
}
