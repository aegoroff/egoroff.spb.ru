#![allow(clippy::unused_async)]
#![allow(non_upper_case_globals)]

use anyhow::Result;
use axum::body::Bytes;
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
use kernel::{graph, resource::Resource};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio_util::io::StreamReader;

use rust_embed::RustEmbed;
use serde::Serialize;

use crate::domain::OperationResult;
use crate::{
    atom,
    body::{Binary, FileReply, Xml},
    domain::{BlogRequest, Navigation, NavigationSection, PageContext, Poster, Uri},
    sitemap,
};

use template::{Index, Search};

use self::error::{ApiError, JsonResult, OperationError, OperationResponse, PageError};

/// Latest posts shown on the home page.
const HOME_POSTS: i32 = 5;

pub mod admin;
pub mod auth;
pub mod blog;
mod error;
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

pub async fn serve_index(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> Result<Response, PageError> {
    let posts = page_context.blog.recent(HOME_POSTS).await?;
    let section = find_section(&page_context, "/")?;
    Ok(Index {
        html_class: "welcome",
        title: kernel::graph::BRAND,
        title_path: "",
        keywords: get_keywords(section),
        meta_description: &section.descr,
        posts,
        apache_docs: &page_context.apache_docs,
        year: get_year(),
    }
    .into_response())
}

pub async fn serve_search(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> Result<Response, PageError> {
    let section = find_section(&page_context, "search")?;
    Ok(Search {
        html_class: "search",
        title: &section.title,
        title_path: "",
        keywords: get_keywords(section),
        meta_description: &section.descr,
        year: get_year(),
    }
    .into_response())
}

pub async fn serve_sitemap(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> Result<Xml<String>, ApiError> {
    let post_ids = page_context.blog.ids().await?;
    Ok(Xml(sitemap::make_site_map(
        &page_context.apache_docs,
        post_ids,
    )?))
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
    let asset = Apache::get(&format!("images/{path}"));
    get_embed(&path, asset)
}

pub async fn serve_storage(
    extract::Path((bucket, path)): extract::Path<(String, String)>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> Result<Response, ApiError> {
    match page_context.file_store.open(&bucket, &path).await? {
        Some(body) => Ok(FileReply::new(body.stream, path, body.length).into_response()),
        None => Err(ApiError::not_found(format!("{bucket}/{path} not found"))),
    }
}

pub fn get_year() -> u32 {
    let now = Utc::now();
    now.year().cast_unsigned()
}

fn get_keywords(section: &SiteSection) -> &str {
    section.keywords.as_ref().map_or("", |v| v)
}

/// Site section by id; a missing section is a broken site map.
fn find_section<'a>(
    page_context: &'a PageContext<'_>,
    id: &str,
) -> Result<&'a SiteSection, PageError> {
    page_context
        .site_graph
        .get_section(id)
        .ok_or_else(|| PageError::internal(format!("no '{id}' section in site graph")))
}

pub async fn serve_navigation(
    Query(query): Query<Uri>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> Response {
    Json(navigation(&page_context.site_graph, &query.uri)).into_response()
}

fn navigation<'a>(site_graph: &'a graph::SiteGraph<'_>, uri: &'a str) -> Navigation<'a> {
    let Some((breadcrumbs, current)) = site_graph.breadcrumbs(uri) else {
        return Navigation::default();
    };

    let root = breadcrumbs[0];
    let optional_breadcrumbs = (uri != graph::SEP).then(|| {
        breadcrumbs
            .iter()
            .map(|s| NavigationSection {
                id: &s.id,
                icon: &s.icon,
                title: &s.title,
                descr: "",
                active: None,
            })
            .collect()
    });
    let sections = root.children.as_ref().map(|children| {
        children
            .iter()
            .map(|s| NavigationSection {
                id: &s.id,
                icon: &s.icon,
                title: &s.title,
                descr: &s.descr,
                active: Some(s.id == *current),
            })
            .collect()
    });

    Navigation {
        sections,
        breadcrumbs: optional_breadcrumbs,
    }
}

fn redirect_response(new_path: &str) -> Response {
    Redirect::permanent(new_path).into_response()
}

fn get_embed(path: &str, asset: Option<rust_embed::EmbeddedFile>) -> Response {
    if let Some(file) = asset {
        Binary::new(file.data, path).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
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

/// Body of a successful admin update or delete.
fn updated() -> Json<OperationResult<'static>> {
    Json(OperationResult { result: "success" })
}

/// Body of a successful admin create.
fn created() -> Json<OperationResult<'static>> {
    Json(OperationResult { result: "created" })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    fn section(id: &str, children: Option<Vec<SiteSection>>) -> SiteSection {
        SiteSection {
            id: id.to_string(),
            icon: format!("{id}-icon"),
            title: format!("{id}-title"),
            descr: format!("{id}-descr"),
            keywords: Some(format!("{id}-keywords")),
            children,
            ..Default::default()
        }
    }

    #[rstest]
    #[case("/", json!({
        "sections": [
            {"id": "a", "icon": "a-icon", "title": "a-title", "descr": "a-descr", "active": false},
            {"id": "b", "icon": "b-icon", "title": "b-title", "descr": "b-descr", "active": false},
        ],
    }))]
    #[case("/a/aa/", json!({
        "sections": [
            {"id": "a", "icon": "a-icon", "title": "a-title", "descr": "a-descr", "active": true},
            {"id": "b", "icon": "b-icon", "title": "b-title", "descr": "b-descr", "active": false},
        ],
        "breadcrumbs": [
            {"id": "/", "icon": "/-icon", "title": "/-title"},
            {"id": "a", "icon": "a-icon", "title": "a-title"},
        ],
    }))]
    fn navigation_serializes_sections_and_breadcrumbs(
        #[case] uri: &str,
        #[case] expected: serde_json::Value,
    ) {
        // arrange
        let aa = section("aa", None);
        let root = section(
            "/",
            Some(vec![section("a", Some(vec![aa])), section("b", None)]),
        );
        let site_graph = graph::SiteGraph::new(&root);

        // act
        let actual = serde_json::to_value(navigation(&site_graph, uri)).unwrap();

        // assert
        assert_eq!(expected, actual);
    }
}
