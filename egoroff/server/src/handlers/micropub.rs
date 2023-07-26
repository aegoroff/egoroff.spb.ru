use axum::{body::Bytes, extract::Multipart, headers::ContentType, http, TypedHeader};
use chrono::Utc;
use kernel::domain::Post;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{
    indie::ME,
    micropub::{MicropubConfig, MicropubForm},
};

use super::*;

const MEDIA_BUCKET: &str = "media";

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
                success_response(Json(config))
            }
            "media-endpoint" => {
                let config = MicropubConfig {
                    media_endpoint,
                    ..Default::default()
                };
                success_response(Json(config))
            }
            "syndicate-to" => {
                let config = MicropubConfig {
                    syndicate_to: Some(vec![]),
                    ..Default::default()
                };
                success_response(Json(config))
            }
            _ => success_response(Empty::new()),
        }
    } else {
        success_response(Empty::new())
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
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(page_context): State<Arc<PageContext<'_>>>,
    body: Bytes,
) -> impl IntoResponse {
    tracing::info!("content type header: {content_type}");
    let form = if let "application/json" = content_type.to_string().to_lowercase().as_str() {
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
        [(http::header::LOCATION, format!("{ME}blog/{post_id}.html"))].into_response(),
    )
}

/// Tries to create a new media or fails with 400 error in case of invalid request.
#[utoipa::path(
    post,
    path = "/micropub/media",
    request_body(content = String, description = "File content", content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "File created successfully"),
        (status = 400, description = "Invalid request syntax", body = MicropubFormError),
        (status = 401, description = "Unauthorized to create post"),
        (status = 500, description = "Server error", body = String),
    ),
    tag = "micropub",
    security(
        ("authorization" = []),
    )
)]
pub async fn serve_media_endpoint_post(
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(page_context): State<Arc<PageContext<'_>>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    tracing::info!("content type header: {content_type}");

    let Some(mut resource) = Resource::new(&page_context.store_uri) else { 
        tracing::error!("Invalid storage uri {}", page_context.store_uri);
        return internal_server_error_response(String::from("Invalid server settings that prevented to reach storage")) 
    };

    let file_name_prefix = Uuid::new_v4();
    let mut file_name = file_name_prefix.to_string();

    let ids : Vec<i64> = if let "multipart/form-data" = content_type.to_string().to_lowercase().as_str() {
        if let Ok(Some(field)) = multipart.next_field().await {
            file_name = format!("{file_name}_{}", field.file_name().unwrap_or_default());
            match read_from_stream(field).await {
                Ok((result, read_bytes)) => {
                    resource
                    .append_path("api")
                    .append_path(MEDIA_BUCKET)
                    .append_path(&file_name);

                    let client = Client::new();
                    let mut form = reqwest::multipart::Form::new();

                    let stream = reqwest::Body::from(result);
                    let part =
                        reqwest::multipart::Part::stream_with_length(stream, read_bytes as u64).file_name(file_name.clone());
                    form = form.part("file", part);
                    let result = client.post(resource.to_string()).multipart(form).send().await;
                    match result {
                        Ok(x) => {
                            match x.json().await {
                                Ok(ids) => ids,
                                Err(e) => return internal_server_error_response(e.to_string()),
                            }
                        }
                        Err(e) => {
                            return internal_server_error_response(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("{e}");
                    return internal_server_error_response(e.to_string());
                }
            }
        } else {
            return bad_request_error_response(
                "no form data received".to_string(),
            );
        }
    } else {
        return bad_request_error_response(
            "expected content-type of multipart/form-data".to_string(),
        );
    };

    tracing::info!("file id: {}", ids[0]);

    (
        StatusCode::CREATED,
        [(http::header::LOCATION, format!("{ME}storage/{MEDIA_BUCKET}/{file_name}"))].into_response(),
    )
}
