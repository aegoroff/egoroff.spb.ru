#![allow(clippy::unused_async)]
#![allow(non_upper_case_globals)]

use anyhow::Result;
use axum::body::{Body, Bytes};
use axum::response::Redirect;
use axum::{
    Extension, Json,
    extract::{self, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{Datelike, Utc};
use futures::{Stream, TryStreamExt};
use futures_util::StreamExt;
use kernel::graph::SiteSection;
use kernel::{
    archive,
    converter::{markdown2html, xml2html},
    domain::{PostsRequest, Storage},
    graph,
    resource::Resource,
};
use std::fmt::Display;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio_util::io::StreamReader;

use reqwest::Client;
use rust_embed::RustEmbed;
use serde::Serialize;

use crate::domain::OperationResult;
use crate::{
    atom,
    body::{Binary, FileReply, Xml},
    domain::{BlogRequest, Error, Navigation, PageContext, Poster, Uri},
    sitemap,
};

use template::{Index, Search};

use self::template::ErrorPage;

pub mod admin;
pub mod auth;
pub mod blog;
pub mod indie;
pub mod micropub;
pub mod portfolio;
mod template;

#[derive(RustEmbed)]
#[folder = "../../static/dist/css"]
struct Css;

#[derive(RustEmbed)]
#[folder = "../../static/dist/js"]
struct Js;

#[derive(RustEmbed)]
#[folder = "../../static/img"]
struct Img;

#[derive(RustEmbed)]
#[folder = "../../static"]
#[include = "*.txt"]
#[include = "*.html"]
#[exclude = "*.json"]
#[exclude = "dist/*"]
#[exclude = "img/*"]
struct Static;

#[derive(RustEmbed)]
#[folder = "../../apache"]
#[exclude = "*.xml"]
#[exclude = "*.xsl"]
#[exclude = "*.dtd"]
struct Apache;

pub async fn serve_index(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let storage = page_context.storage.lock().await;
    let result = archive::get_small_posts(storage, 5, None);

    let blog_posts = match result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("{e:#?}");
            return internal_server_error_page();
        }
    };

    match portfolio::read_apache_documents(&page_context.base_path) {
        Ok(docs) => {
            if let Some(section) = page_context.site_graph.get_section("/") {
                Index {
                    html_class: "welcome",
                    title: kernel::graph::BRAND,
                    title_path: "",
                    keywords: get_keywords(section),
                    meta_description: &section.descr,
                    posts: blog_posts.result,
                    apache_docs: docs,
                    flashed_messages: vec![],
                    year: get_year(),
                }
                .into_response()
            } else {
                internal_server_error_page()
            }
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            internal_server_error_page()
        }
    }
}

pub async fn serve_search(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    if let Some(section) = page_context.site_graph.get_section("search") {
        Search {
            html_class: "search",
            title: &section.title,
            title_path: "",
            keywords: get_keywords(section),
            meta_description: &section.descr,
            flashed_messages: vec![],
            config: &page_context.site_config,
            year: get_year(),
        }
        .into_response()
    } else {
        tracing::error!("no search section found in graph");
        internal_server_error_page()
    }
}

pub async fn serve_sitemap(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let apache_documents = portfolio::read_apache_documents(&page_context.base_path);

    let apache_documents = match apache_documents {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("{e:#?}");
            let content = format!("<?xml version=\"1.0\"?><error>{e}</error>");
            return internal_server_error_response(Xml(content));
        }
    };

    let storage = page_context.storage.lock().await;
    let post_ids = match storage.get_posts_ids() {
        Ok(ids) => ids,
        Err(e) => {
            return internal_server_error_response(format!(
                "<?xml version=\"1.0\"?><error>{e}</error>"
            ));
        }
    };
    let xml = match sitemap::make_site_map(apache_documents, post_ids) {
        Ok(xml) => xml,
        Err(e) => {
            return internal_server_error_response(format!(
                "<?xml version=\"1.0\"?><error>{e}</error>"
            ));
        }
    };
    success_response(Xml(xml))
}

pub async fn serve_js(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    let path = path.as_str();
    let asset = Js::get(path);
    get_embed(path, asset)
}

pub async fn serve_root(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    let path = path.as_str();
    let asset = if path == "favicon.ico" {
        Img::get(path)
    } else {
        Static::get(path)
    };
    get_embed(path, asset)
}

pub async fn serve_css(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    let path = path.as_str();
    let asset = Css::get(path);
    get_embed(path, asset)
}

pub async fn serve_img(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    let path = path.as_str();
    let asset = Img::get(path);
    get_embed(path, asset)
}

pub async fn serve_apache(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    let path = path.as_str();
    let asset = Apache::get(path);
    get_embed(path, asset)
}

pub async fn serve_apache_images(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    let path = path.as_str();
    let relative_path = PathBuf::from("images");
    let relative_path = relative_path.join(path);
    let relative_path = relative_path.as_os_str().to_str().unwrap_or_default();
    let asset = Apache::get(relative_path);
    get_embed(path, asset)
}

pub async fn serve_storage(
    extract::Path((bucket, path)): extract::Path<(String, String)>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let Some(mut resource) = Resource::new(&page_context.store_uri) else {
        tracing::error!("Invalid storage uri {}", page_context.store_uri);
        return internal_server_error_response(String::from(
            "Invalid server settings that prevented to reach storage",
        ));
    };

    // to prevent path traversal attacks
    if !is_safe_path_segment(&bucket) || !is_safe_path_segment(&path) {
        return bad_request_error_response(format!(
            "invalid bucket: '{bucket}' or path: '{path}' detected."
        ));
    }

    resource
        .append_path("api")
        .append_path(&bucket)
        .append_path(&path);

    let client = Client::new();

    match client.get(resource.to_string()).send().await {
        Ok(response) => match response.error_for_status() {
            Ok(r) => {
                let headers = r.headers();
                let len = get_content_length(headers);
                success_response(FileReply::new(r.bytes_stream(), path, len))
            }
            Err(e) => {
                tracing::error!("{e:#?}");
                not_found_response(format!("{bucket}/{path} not found"))
            }
        },
        Err(e) => {
            tracing::error!("{e:#?}");
            bad_request_error_response(Body::empty())
        }
    }
}

pub fn get_year() -> u32 {
    let now = Utc::now();
    now.year() as u32
}

fn get_keywords(section: &SiteSection) -> &str {
    section.keywords.as_ref().map_or("", |v| v)
}

/// makes HTTP (OK) response code 200
fn success_response<R: IntoResponse>(r: R) -> (StatusCode, Response) {
    (StatusCode::OK, r.into_response())
}

/// makes HTTP (NOT FOUND) response code 404
fn not_found_response<R: IntoResponse>(r: R) -> (StatusCode, Response) {
    (StatusCode::NOT_FOUND, r.into_response())
}

/// makes HTTP (INTERNAL SERVER ERROR) response code 500
fn internal_server_error_response<R: IntoResponse>(r: R) -> (StatusCode, Response) {
    (StatusCode::INTERNAL_SERVER_ERROR, r.into_response())
}

/// makes HTTP (BAD REQUEST) response code 400
fn bad_request_error_response<R: IntoResponse>(r: R) -> (StatusCode, Response) {
    (StatusCode::BAD_REQUEST, r.into_response())
}

/// makes HTTP (UNAUTHORIZED) response code 401
fn unauthorized_response<R: IntoResponse>(r: R) -> (StatusCode, Response) {
    (StatusCode::UNAUTHORIZED, r.into_response())
}

fn get_content_length(headers: &axum::http::HeaderMap) -> Option<i64> {
    let len_header = headers.get("content-length")?;
    let val = len_header.to_str().ok()?;
    val.parse().ok()
}

pub async fn serve_navigation(
    Query(query): Query<Uri>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let q = query.uri;

    let Some((breadcrumbs, current)) = page_context.site_graph.breadcrumbs(&q) else {
        return Json(Navigation {
            ..Default::default()
        });
    };

    let root = breadcrumbs[0];
    let optional_breadcrumbs = if q == graph::SEP {
        None
    } else {
        Some(
            breadcrumbs
                .into_iter()
                .map(|s| SiteSection {
                    id: s.id.clone(),
                    icon: s.icon.clone(),
                    title: s.title.clone(),
                    ..Default::default()
                })
                .collect(),
        )
    };

    Json(Navigation {
        sections: root.clone_children(current),
        breadcrumbs: optional_breadcrumbs,
    })
}

fn is_safe_path_segment(segment: &str) -> bool {
    if segment.is_empty() || segment == ".." || segment.contains('/') || segment.contains(':') {
        return false;
    }

    segment
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn not_found_page() -> Response {
    not_found_response(error_page_response("404")).into_response()
}

fn internal_server_error_page() -> Response {
    internal_server_error_response(error_page_response("500")).into_response()
}

fn redirect_response(new_path: &str) -> Response {
    (
        StatusCode::PERMANENT_REDIRECT,
        Redirect::permanent(new_path).into_response(),
    )
        .into_response()
}

fn error_page_response(code: &str) -> Response {
    let error = Error {
        code: code.to_string(),
        ..Default::default()
    };
    ErrorPage {
        html_class: "",
        title: code,
        title_path: "",
        keywords: "",
        meta_description: "",
        error,
        flashed_messages: vec![],
        year: get_year(),
    }
    .into_response()
}

fn get_embed(path: &str, asset: Option<rust_embed::EmbeddedFile>) -> impl IntoResponse + use<> {
    if let Some(file) = asset {
        success_response(Binary::new(file.data, path))
    } else {
        not_found_response(Body::empty())
    }
}

fn make_json_response<T: Default + Serialize>(result: Result<T>) -> impl IntoResponse {
    match result {
        Ok(ar) => success_response(Json(ar)),
        Err(e) => {
            tracing::error!("Execution error: {e:#?}");
            let r: T = Default::default();
            internal_server_error_response(Json(r))
        }
    }
}

async fn read_from_stream<S, E>(stream: S) -> Result<(Vec<u8>, usize)>
where
    S: Stream<Item = Result<Bytes, E>> + StreamExt,
    E: Sync + std::error::Error + Send + 'static,
{
    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::other(err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);
    let mut buffer = Vec::new();

    let copied_bytes = tokio::io::copy(&mut body_reader, &mut buffer).await?;
    Ok((buffer, copied_bytes as usize))
}

fn updated_response<T, E: Display>(result: Result<T, E>) -> impl IntoResponse {
    if let Err(e) = result {
        let error = format!("{e}");
        internal_server_error_response(Json(OperationResult { result: &error }))
    } else {
        success_response(Json(OperationResult { result: "success" }))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("123", 123)]
    #[case("0", 0)]
    #[case("-1", -1)]
    #[case("8000000000", 8_000_000_000)]
    #[trace]
    fn get_content_length_positive_tests(#[case] test_data: &str, #[case] expected: i64) {
        // arrange
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("host", "example.com".parse().unwrap());
        headers.insert("content-length", test_data.parse().unwrap());

        // act
        let actual = get_content_length(&headers);

        // assert
        assert_eq!(Some(expected), actual);
    }

    #[test]
    fn get_content_length_no_header() {
        // arrange
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("host", "example.com".parse().unwrap());

        // act
        let actual = get_content_length(&headers);

        // assert
        assert!(actual.is_none());
    }

    #[test]
    fn get_content_length_incorrect_header() {
        // arrange
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("host", "example.com".parse().unwrap());
        headers.insert("content-length", "www".parse().unwrap());

        // act
        let actual = get_content_length(&headers);

        // assert
        assert!(actual.is_none());
    }

    #[test]
    fn get_content_length_header_in_other_case() {
        // arrange
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("host", "example.com".parse().unwrap());
        headers.insert("Content-Length", "123".parse().unwrap());

        // act
        let actual = get_content_length(&headers);

        // assert
        assert_eq!(Some(123), actual);
    }
}
