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
use kernel::{domain::Storage, graph, resource::Resource};
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

/// Latest posts shown on the home page.
const HOME_POSTS: i32 = 5;

pub mod admin;
pub mod auth;
pub mod blog;
pub mod indie;
pub mod micropub;
pub mod portfolio;
pub mod search;
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
    let blog_posts = match page_context.blog.recent(HOME_POSTS).await {
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
                    posts: blog_posts,
                    apache_docs: docs,
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

    let post_ids = match page_context.blog.ids().await {
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
    match page_context.file_store.open(&bucket, &path).await {
        Ok(Some(body)) => success_response(FileReply::new(body.stream, path, body.length)),
        Ok(None) => not_found_response(format!("{bucket}/{path} not found")),
        Err(e) => {
            tracing::error!("{e:#?}");
            internal_server_error_response(Body::empty())
        }
    }
}

pub fn get_year() -> u32 {
    let now = Utc::now();
    now.year().cast_unsigned()
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

fn not_found_page() -> Response {
    not_found_response(error_page_response("404")).into_response()
}

fn internal_server_error_page() -> Response {
    internal_server_error_response(error_page_response("500")).into_response()
}

fn redirect_response(new_path: &str) -> Response {
    Redirect::permanent(new_path).into_response()
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

async fn read_from_stream<S, E>(stream: S) -> Result<Vec<u8>>
where
    S: Stream<Item = Result<Bytes, E>> + StreamExt,
    E: Sync + std::error::Error + Send + 'static,
{
    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::other(err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);
    let mut buffer = Vec::new();

    tokio::io::copy(&mut body_reader, &mut buffer).await?;
    Ok(buffer)
}

fn updated_response<T, E: Display>(result: Result<T, E>) -> impl IntoResponse {
    if let Err(e) = result {
        let error = format!("{e}");
        internal_server_error_response(Json(OperationResult { result: &error }))
    } else {
        success_response(Json(OperationResult { result: "success" }))
    }
}

fn created_response<T, E: Display>(result: Result<T, E>) -> impl IntoResponse {
    if let Err(e) = result {
        let error = format!("{e}");
        internal_server_error_response(Json(OperationResult { result: &error }))
    } else {
        success_response(Json(OperationResult { result: "created" }))
    }
}
