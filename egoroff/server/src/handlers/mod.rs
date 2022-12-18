use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;

use axum::{
    body::{Empty, Full},
    extract::{self, Query},
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect},
    Extension, Json,
};
use axum_sessions::extractors::{ReadableSession, WritableSession};
use kernel::{
    archive,
    converter::{markdown2html, xml2html},
    domain::{PostsRequest, Storage, User},
    graph::SiteSection,
    sqlite::{Mode, Sqlite},
};

use rust_embed::RustEmbed;
use tera::{Context, Tera};

use crate::{
    atom,
    body::Xml,
    domain::{BlogRequest, Error, Navigation, PageContext, Poster, Uri},
    sitemap,
};

pub mod auth;
pub mod blog;
pub mod portfolio;

#[cfg(debug_assertions)]
const MODE: &str = "debug";

#[cfg(not(debug_assertions))]
const MODE: &str = "release";

const TITLE_KEY: &str = "title";

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
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let req = PostsRequest {
        ..Default::default()
    };

    let result = archive::get_posts(&page_context.storage_path, 5, req);

    let section = page_context.site_graph.get_section("/").unwrap();
    let mut context = Context::new();
    context.insert("html_class", "welcome");
    context.insert(TITLE_KEY, "egoroff.spb.ru");
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("config", &page_context.site_config);
    context.insert("ctx", "");
    context.insert("posts", &result.result);

    match portfolio::apache_documents(&page_context.base_path) {
        Ok(docs) => {
            context.insert("apache_docs", &docs);
            (
                StatusCode::OK,
                serve_page(&context, "welcome.html", &page_context.tera),
            )
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                make_500_page(&mut context, &page_context.tera),
            )
        }
    }
}

pub async fn serve_search(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let section = page_context.site_graph.get_section("search").unwrap();

    let mut context = Context::new();
    context.insert("html_class", "search");
    context.insert(TITLE_KEY, &section.title);
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("ctx", "");
    context.insert("config", &page_context.site_config);

    serve_page(&context, "search.html", &page_context.tera)
}

pub async fn serve_sitemap(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let apache_documents = portfolio::apache_documents(&page_context.base_path);

    let apache_documents = match apache_documents {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("{e:#?}");
            let content = format!("<?xml version=\"1.0\"?><error>{}</error>", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Xml(content));
        }
    };

    let storage = Sqlite::open(&page_context.storage_path, Mode::ReadOnly).unwrap();
    let post_ids = storage.get_posts_ids().unwrap();
    let xml = sitemap::make_site_map(apache_documents, post_ids).unwrap();
    (StatusCode::OK, Xml(xml))
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

// this handler gets called if the query deserializes into `Info` successfully
// otherwise a 400 Bad Request error response is returned
pub async fn navigation(
    extract::Query(query): extract::Query<Uri>,
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let q = query.uri;

    let (breadcrumbs, current) = page_context.site_graph.breadcrumbs(&q);

    let breadcrumbs = if q != "/" { Some(breadcrumbs) } else { None };

    match page_context.site_graph.get_section("/") {
        Some(r) => Json(Navigation {
            sections: activate_section(r.children, &current),
            breadcrumbs,
        }),
        None => Json(Navigation {
            ..Default::default()
        }),
    }
}

fn make_404_page(context: &mut Context, tera: &Tera) -> Html<String> {
    make_error_page(context, "404", tera)
}

fn make_500_page(context: &mut Context, tera: &Tera) -> Html<String> {
    make_error_page(context, "500", tera)
}

fn make_error_page(context: &mut Context, code: &str, tera: &Tera) -> Html<String> {
    let error = Error {
        code: code.to_string(),
        ..Default::default()
    };
    if context.contains_key(TITLE_KEY) {
        context.remove(TITLE_KEY);
    }
    context.insert(TITLE_KEY, code);
    context.insert("error", &error);
    serve_page(context, "error.html", tera)
}

fn serve_page(context: &Context, template_name: &str, tera: &Tera) -> Html<String> {
    let index = tera.render(template_name, context);
    match index {
        Ok(content) => Html(content),
        Err(err) => {
            tracing::error!("Server error: {err}");
            Html(format!("{:#?}", err))
        }
    }
}

fn get_embed(path: &str, asset: Option<rust_embed::EmbeddedFile>) -> impl IntoResponse {
    if let Some(file) = asset {
        let mut res = Full::from(file.data).into_response();
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        res.headers_mut().insert(
            "content-type",
            HeaderValue::from_str(mime.as_ref()).unwrap(),
        );
        (StatusCode::OK, res)
    } else {
        (StatusCode::NOT_FOUND, Empty::new().into_response())
    }
}

fn activate_section(sections: Option<Vec<SiteSection>>, current: &str) -> Option<Vec<SiteSection>> {
    if let Some(sections) = sections {
        Some(
            sections
                .into_iter()
                .map(|mut s| {
                    s.active = Some(s.id == current);
                    s
                })
                .collect(),
        )
    } else {
        sections
    }
}
