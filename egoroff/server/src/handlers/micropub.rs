#![allow(clippy::module_name_repetitions)]

use anyhow::Context;
use axum::{body::Bytes, extract::Multipart, http};
use axum_extra::{TypedHeader, headers::ContentType};
use mime_guess::mime::{self, Mime};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    indie::ME,
    micropub::{MicropubConfig, MicropubForm, MicropubFormError, MicropubSource, parse_post_url},
};

use super::*;

const MEDIA_BUCKET: &str = "media";

#[derive(Deserialize, Serialize, IntoParams)]
pub struct MicropubRequest {
    pub q: Option<String>,
    pub url: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct File {
    pub id: i64,
    pub path: String,
    pub bucket: String,
    pub size: usize,
}

/// Response containing a URL to the uploaded media.
///
/// This struct is returned by the micropub media endpoint and
/// represents the location where the media can be accessed.
#[derive(Serialize, ToSchema)]
pub struct MediaResponse {
    /// The absolute URL of the stored media resource.
    ///
    /// This URL can be used by clients to retrieve the media content.
    pub url: String,
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
pub async fn serve_index_get(
    Query(query): Query<MicropubRequest>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
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
            "source" => {
                let Some(post_url) = query.url else {
                    return bad_request_error_response(
                        "url query parameter is required for source",
                    );
                };
                let Some(post_id) = parse_post_url(ME, &post_url) else {
                    return bad_request_error_response("invalid post url");
                };

                let storage = page_context.storage.lock().await;
                let post = match storage.get_post(post_id) {
                    Ok(post) => post,
                    Err(e) => {
                        tracing::error!("micropub source post {post_id} not found: {e:#?}");
                        return not_found_response("post not found");
                    }
                };

                let source = MicropubSource::from_post(&post);
                success_response(Json(source))
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
            _ => success_response(Body::empty()),
        }
    } else {
        success_response(Body::empty())
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
    let form = if is_media_type(content_type, &mime::APPLICATION_JSON) {
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

    let mut storage = page_context.storage.lock().await;
    let post_id = match storage.next_post_id() {
        Ok(id) => id,
        Err(e) => return internal_server_error_response(e.to_string()),
    };
    tracing::info!("content type: {:?}", form.content_type);
    let post = form.to_post(post_id);
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
        (status = 401, description = "Unauthorized to create media"),
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
        return internal_server_error_response(String::from(
            "Invalid server settings that prevented to reach storage",
        ));
    };

    if !is_media_type(content_type, &mime::MULTIPART_FORM_DATA) {
        return bad_request_error_response("expected content-type of multipart/form-data");
    }

    let Ok(Some(field)) = multipart.next_field().await else {
        return bad_request_error_response("no form data received");
    };

    let file_name = media_storage_file_name(Uuid::new_v4(), field.file_name());
    let (data, read_bytes) = match read_from_stream(field).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("{e}");
            return internal_server_error_response(e.to_string());
        }
    };

    resource
        .append_path("api")
        .append_path(MEDIA_BUCKET)
        .append_path(&file_name);

    match upload_media(&resource, &file_name, data, read_bytes).await {
        Ok(id) => tracing::info!("file id: {id}"),
        Err(e) => {
            tracing::error!("media upload failed: {e:#}");
            return internal_server_error_response(e.to_string());
        }
    }

    (
        StatusCode::CREATED,
        [(
            http::header::LOCATION,
            format!("{ME}storage/{MEDIA_BUCKET}/{file_name}"),
        )]
        .into_response(),
    )
}

/// Gets last inserted media uri
#[utoipa::path(
    get,
    path = "/micropub/media",
    params(
        MicropubRequest
    ),
    responses(
        (status = 200, description = "Last uri get successfully", body = MediaResponse),
        (status = 400, description = "Invalid request syntax", body = MicropubFormError),
        (status = 401, description = "Unauthorized to get last inserted media file"),
        (status = 404, description = "No last inserted file found"),
        (status = 500, description = "Server error", body = String),
    ),
    tag = "micropub",
    security(
        ("authorization" = []),
    )
)]
pub async fn serve_media_endpoint_get(
    State(page_context): State<Arc<PageContext<'_>>>,
    Query(req): Query<MicropubRequest>,
) -> impl IntoResponse {
    if let Some(q) = req.q {
        if q != "last" {
            return bad_request_error_response(format!(
                "Invalid query. Must be last but was '{q}'"
            ));
        }
    } else {
        return bad_request_error_response(String::from("No query"));
    }

    let Some(mut resource) = Resource::new(&page_context.store_uri) else {
        return internal_server_error_response(String::from(
            "Invalid server settings that prevented to reach storage",
        ));
    };

    resource
        .append_path("api")
        .append_path(MEDIA_BUCKET)
        .append_path("last");
    let client = Client::new();
    let result = client.get(resource.to_string()).send().await;
    match result {
        Ok(x) => {
            tracing::info!("Response status: {}", x.status());
            match x.json::<File>().await {
                Ok(file) => {
                    let response = MediaResponse {
                        url: format!("{ME}storage/{MEDIA_BUCKET}/{}", file.path),
                    };
                    success_response(Json(response))
                }
                Err(e) => internal_server_error_response(e.to_string()),
            }
        }
        Err(e) => internal_server_error_response(e.to_string()),
    }
}

/// Uploads a file to the file store and returns the id it was stored under.
async fn upload_media(
    resource: &Resource,
    file_name: &str,
    data: Vec<u8>,
    len: usize,
) -> Result<i64> {
    let part = reqwest::multipart::Part::stream_with_length(reqwest::Body::from(data), len as u64)
        .file_name(file_name.to_owned());
    let form = reqwest::multipart::Form::new().part("file", part);

    let ids: Vec<i64> = Client::new()
        .post(resource.to_string())
        .multipart(form)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    first_file_id(&ids)
}

fn first_file_id(ids: &[i64]) -> Result<i64> {
    ids.first()
        .copied()
        .context("file store returned no file id")
}

/// Compares the media type of a `Content-Type` header with `expected`,
/// ignoring parameters such as `charset` or `boundary`.
fn is_media_type(content_type: ContentType, expected: &Mime) -> bool {
    Mime::from(content_type).essence_str() == expected.essence_str()
}

/// Builds a storage object name that cannot escape the media bucket.
///
/// Client-supplied filenames may contain path separators or `..` segments.
/// Only a safe ASCII alphanumeric extension is preserved from the basename.
fn media_storage_file_name(id: Uuid, client_file_name: Option<&str>) -> String {
    let id = id.to_string();
    let Some(client_file_name) = client_file_name.filter(|name| !name.is_empty()) else {
        return id;
    };

    let basename = client_file_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(client_file_name);

    let Some(extension) = basename
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| !ext.is_empty() && ext.chars().all(|c| c.is_ascii_alphanumeric()))
    else {
        return id;
    };

    format!("{id}.{extension}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::{first_file_id, is_media_type, media_storage_file_name};
    use axum_extra::headers::ContentType;
    use mime_guess::mime;
    use rstest::rstest;
    use uuid::Uuid;

    const ID: Uuid = Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);

    #[rstest]
    #[case(None, "01234567-89ab-cdef-0123-456789abcdef")]
    #[case(Some(""), "01234567-89ab-cdef-0123-456789abcdef")]
    #[case(Some("photo.jpg"), "01234567-89ab-cdef-0123-456789abcdef.jpg")]
    #[case(Some("photo.JPEG"), "01234567-89ab-cdef-0123-456789abcdef.JPEG")]
    #[case(Some("../../etc/passwd"), "01234567-89ab-cdef-0123-456789abcdef")]
    #[case(
        Some("../other-bucket/x.png"),
        "01234567-89ab-cdef-0123-456789abcdef.png"
    )]
    #[case(Some(r"..\windows\x.png"), "01234567-89ab-cdef-0123-456789abcdef.png")]
    #[case(Some("archive.tar.gz"), "01234567-89ab-cdef-0123-456789abcdef.gz")]
    #[case(Some("no-extension"), "01234567-89ab-cdef-0123-456789abcdef")]
    #[case(Some("bad.ext-1"), "01234567-89ab-cdef-0123-456789abcdef")]
    #[case(Some("bad."), "01234567-89ab-cdef-0123-456789abcdef")]
    fn media_storage_file_name_tests(
        #[case] client_file_name: Option<&str>,
        #[case] expected: &str,
    ) {
        // Arrange / Act
        let actual = media_storage_file_name(ID, client_file_name);

        // Assert
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("application/json", true)]
    #[case("application/json; charset=utf-8", true)]
    #[case("Application/JSON", true)]
    #[case("application/x-www-form-urlencoded", false)]
    #[case("text/json", false)]
    fn is_media_type_json_tests(#[case] header: &str, #[case] expected: bool) {
        // Arrange
        let content_type: ContentType = header.parse().unwrap();

        // Act
        let actual = is_media_type(content_type, &mime::APPLICATION_JSON);

        // Assert
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("multipart/form-data", true)]
    #[case("multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxk", true)]
    #[case("multipart/mixed; boundary=abc", false)]
    #[case("application/octet-stream", false)]
    fn is_media_type_multipart_tests(#[case] header: &str, #[case] expected: bool) {
        // Arrange
        let content_type: ContentType = header.parse().unwrap();

        // Act
        let actual = is_media_type(content_type, &mime::MULTIPART_FORM_DATA);

        // Assert
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(&[42], 42)]
    #[case(&[42, 7], 42)]
    fn first_file_id_returns_first(#[case] ids: &[i64], #[case] expected: i64) {
        // Arrange / Act
        let actual = first_file_id(ids).unwrap();

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn first_file_id_empty_is_error() {
        // Arrange
        let ids: &[i64] = &[];

        // Act
        let actual = first_file_id(ids);

        // Assert
        assert!(actual.is_err());
    }
}
