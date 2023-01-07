use std::{path::PathBuf, sync::Arc};

use axum::{body::Empty, extract::Query, http, response::IntoResponse, Extension};
use axum_sessions::extractors::WritableSession;
use chrono::{Duration, Utc};
use http::StatusCode;
use kernel::resource::Resource;

use crate::{
    body::Redirect,
    domain::PageContext,
    indie::{generate_jwt, Claims, IndieQuery},
};

const ME: &str = "https://www.egoroff.spb.ru/";
const INDIE_TOKEN_KEY: &str = "indie_token";

pub async fn serve_auth(
    Query(query): Query<IndieQuery>,
    Extension(page_context): Extension<Arc<PageContext>>,
    mut session: WritableSession,
) -> impl IntoResponse {
    let private_key_path = PathBuf::from(&page_context.certs_path).join("egoroffspbrupri.pem");

    if let Some(redirect) = query.redirect_uri {
        if let Some(client_id) = query.client_id {
            if redirect.starts_with(&client_id) {
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
                        session.insert(INDIE_TOKEN_KEY, token).unwrap();
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
            } else {
                tracing::error!("invalid redirect uri '{redirect}' that doest start from client id '{client_id}'");
                (StatusCode::BAD_REQUEST, Empty::new().into_response())
            }
        } else {
            tracing::error!("No client id");
            (StatusCode::BAD_REQUEST, Empty::new().into_response())
        }
    } else {
        (StatusCode::BAD_REQUEST, Empty::new().into_response())
    }
}
