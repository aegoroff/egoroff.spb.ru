#![allow(clippy::module_name_repetitions)]

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
) -> Result<Response, ApiError> {
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
                Ok(Json(config).into_response())
            }
            "source" => {
                let post_url = query.url.ok_or_else(|| {
                    ApiError::bad_request("url query parameter is required for source")
                })?;
                let post_id = parse_post_url(ME, &post_url)
                    .ok_or_else(|| ApiError::bad_request("invalid post url"))?;
                let post = page_context
                    .blog
                    .draft(post_id)
                    .await?
                    .ok_or_else(|| ApiError::not_found("post not found"))?;
                Ok(Json(MicropubSource::from_post(&post)).into_response())
            }
            "media-endpoint" => {
                let config = MicropubConfig {
                    media_endpoint,
                    ..Default::default()
                };
                Ok(Json(config).into_response())
            }
            "syndicate-to" => {
                let config = MicropubConfig {
                    syndicate_to: Some(vec![]),
                    ..Default::default()
                };
                Ok(Json(config).into_response())
            }
            _ => Ok(StatusCode::OK.into_response()),
        }
    } else {
        Ok(StatusCode::OK.into_response())
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
) -> Result<Response, ApiError> {
    tracing::info!("content type header: {content_type}");
    let form = if is_media_type(content_type, &mime::APPLICATION_JSON) {
        MicropubForm::from_json_bytes(&body.slice(..))
    } else {
        // x-www-form-urlencoded
        MicropubForm::from_form_bytes(&body.slice(..))
    };
    let form = form.map_err(|e| ApiError::bad_request(e.to_string()))?;

    tracing::info!("content type: {:?}", form.content_type);
    let post_id = page_context.blog.create(|id| form.to_post(id)).await?;
    Ok((
        StatusCode::CREATED,
        [(http::header::LOCATION, format!("{ME}blog/{post_id}.html"))],
    )
        .into_response())
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
) -> Result<Response, ApiError> {
    tracing::info!("content type header: {content_type}");

    if !is_media_type(content_type, &mime::MULTIPART_FORM_DATA) {
        return Err(ApiError::bad_request(
            "expected content-type of multipart/form-data",
        ));
    }

    let Ok(Some(field)) = multipart.next_field().await else {
        return Err(ApiError::bad_request("no form data received"));
    };

    let file_name = media_storage_file_name(Uuid::new_v4(), field.file_name());
    let data = read_from_stream(field).await?;
    let id = page_context
        .file_store
        .upload(MEDIA_BUCKET, &file_name, data)
        .await?;
    tracing::info!("file id: {id}");

    Ok((
        StatusCode::CREATED,
        [(
            http::header::LOCATION,
            format!("{ME}storage/{MEDIA_BUCKET}/{file_name}"),
        )],
    )
        .into_response())
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
) -> Result<Json<MediaResponse>, ApiError> {
    match req.q.as_deref() {
        Some("last") => {}
        Some(q) => {
            return Err(ApiError::bad_request(format!(
                "Invalid query. Must be last but was '{q}'"
            )));
        }
        None => return Err(ApiError::bad_request("No query")),
    }

    let file = page_context.file_store.last(MEDIA_BUCKET).await?;
    Ok(Json(MediaResponse {
        url: format!("{ME}storage/{MEDIA_BUCKET}/{}", file.path),
    }))
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
    use super::{is_media_type, media_storage_file_name};
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
}
