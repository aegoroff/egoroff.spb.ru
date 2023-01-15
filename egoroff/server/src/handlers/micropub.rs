use axum::{body::Bytes, http};
use chrono::Utc;
use kernel::domain::Post;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    indie::ME,
    micropub::{MicropubConfig, MicropubForm},
};

use super::*;

#[derive(Deserialize, Serialize, IntoParams)]
pub struct MicropubRequest {
    pub q: Option<String>,
    pub url: Option<String>,
}

/// Gets micropub endpoint configuration to find out it's capabilities
#[utoipa::path(
    get,
    path = "/micropub/",
    responses(
        (status = 200, description = "Configuration read successfully", body = MicropubConfig),
        (status = 401, description = "Unauthorized to read configuration"),
    ),
    params(
        MicropubRequest
    ),
    tag = "micropub",
    security(
        ("authorization" = [])
    )
)]
pub async fn serve_index_get(Query(query): Query<MicropubRequest>) -> impl IntoResponse {
    if let Some(q) = query.q {
        let media_endpoint = Some(format!("{ME}micropub/media"));
        match q.as_str() {
            "config" => {
                let config = MicropubConfig {
                    q: Some(vec![
                        "config".to_string(),
                        "media-endpoint".to_string(),
                        "source".to_string(),
                        "syndicate-to".to_string(),
                    ]),
                    media_endpoint,
                    syndicate_to: Some(vec![]),
                };
                (StatusCode::OK, Json(config).into_response())
            }
            "media-endpoint" => {
                let config = MicropubConfig {
                    media_endpoint,
                    ..Default::default()
                };
                (StatusCode::OK, Json(config).into_response())
            }
            "syndicate-to" => {
                let config = MicropubConfig {
                    syndicate_to: Some(vec![]),
                    ..Default::default()
                };
                (StatusCode::OK, Json(config).into_response())
            }
            _ => (StatusCode::OK, Empty::new().into_response()),
        }
    } else {
        (StatusCode::OK, Empty::new().into_response())
    }
}

/// Tries to create a new Post or fails with 400 error in case of invalid request.
#[utoipa::path(
    post,
    path = "/micropub/",
    request_body(content = String, description = "Post content", content_type = "application/json"),
    responses(
        (status = 201, description = "Post created successfully"),
        (status = 400, description = "Invalid request syntax", body = MicropubFormError),
        (status = 401, description = "Unauthorized to create post"),
        (status = 500, description = "Server error", body = String),
    ),
    tag = "micropub",
    security(
        ("authorization" = []),
    )
)]
pub async fn serve_index_post(
    headers: http::header::HeaderMap,
    Extension(page_context): Extension<Arc<PageContext>>,
    body: Bytes,
) -> impl IntoResponse {
    let content_type = headers.get("Content-Type");
    let ct: String = content_type
        .map(move |c| c.to_str().unwrap_or("x-www-form-url-encoded").into())
        .unwrap_or_else(|| "x-www-form-url-encoded".into());

    let form = if let "application/json" = ct.to_lowercase().as_str() {
        MicropubForm::from_json_bytes(&body.slice(..))
    } else {
        // x-www-form-urlencoded
        MicropubForm::from_form_bytes(&body.slice(..))
    };
    let form = match form {
        Ok(f) => f,
        Err(e) => {
            return bad_request_error_response(e.to_string());
        }
    };
    let created = Utc::now();

    let mut storage = page_context.storage.lock().await;
    let post_id = match storage.next_post_id() {
        Ok(id) => id,
        Err(e) => return internal_server_error_response(e.to_string()),
    };
    let content_type = form.content_type.unwrap_or_default();
    tracing::info!("content type: {content_type}");
    let markdown = content_type == "markdown" || content_type.is_empty();
    let post = Post {
        created,
        modified: created,
        id: post_id,
        title: form.name.unwrap_or_default(),
        short_text: String::new(),
        text: form.content,
        markdown,
        is_public: false,
        tags: vec![],
    };
    if let Err(e) = storage.upsert_post(post) {
        return internal_server_error_response(e.to_string());
    }
    (
        StatusCode::CREATED,
        [(
            http::header::LOCATION,
            format!("{}blog/{}.html", ME, post_id),
        )]
        .into_response(),
    )
}
