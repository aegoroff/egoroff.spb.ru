use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;

use axum::{
    body::Empty,
    extract::{self, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect},
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
    body::{Binary, Html, Xml},
    domain::{BlogRequest, Error, Navigation, PageContext, Poster, Uri},
    sitemap,
};

pub mod admin;
pub mod auth;
pub mod blog;
pub mod portfolio;

const TITLE_KEY: &str = "title";
const TITLE_PATH_KEY: &str = "title_path";
const HTML_CLASS_KEY: &str = "html_class";
const KEYWORDS_KEY: &str = "keywords";
const META_KEY: &str = "meta_description";
const CONFIG_KEY: &str = "config";
const APACHE_DOCS_KEY: &str = "apache_docs";

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

    let result = archive::get_small_posts(&page_context.storage_path, 5, req);

    let section = page_context.site_graph.get_section("/").unwrap();
    let mut context = Context::new();
    context.insert(HTML_CLASS_KEY, "welcome");
    context.insert(TITLE_KEY, "egoroff.spb.ru");
    context.insert(KEYWORDS_KEY, &section.keywords);
    context.insert(META_KEY, &section.descr);
    context.insert(CONFIG_KEY, &page_context.site_config);
    context.insert("posts", &result.result);

    match portfolio::read_apache_documents(&page_context.base_path) {
        Ok(docs) => {
            context.insert(APACHE_DOCS_KEY, &docs);
            serve_page(&context, "welcome.html", &page_context.tera)
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            make_500_page(&mut context, &page_context.tera)
        }
    }
}

pub async fn serve_search(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let section = page_context.site_graph.get_section("search").unwrap();

    let mut context = Context::new();
    context.insert(HTML_CLASS_KEY, "search");
    context.insert(TITLE_KEY, &section.title);
    context.insert(KEYWORDS_KEY, &section.keywords);
    context.insert(META_KEY, &section.descr);
    context.insert(CONFIG_KEY, &page_context.site_config);

    serve_page(&context, "search.html", &page_context.tera)
}

pub async fn serve_sitemap(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let apache_documents = portfolio::read_apache_documents(&page_context.base_path);

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

fn make_404_page(context: &mut Context, tera: &Tera) -> (StatusCode, Html<String>) {
    (StatusCode::NOT_FOUND, make_error_page(context, "404", tera))
}

fn make_500_page(context: &mut Context, tera: &Tera) -> (StatusCode, Html<String>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        make_error_page(context, "500", tera),
    )
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
    let index = tera.render("error.html", context);
    match index {
        Ok(content) => Html(content),
        Err(err) => {
            tracing::error!("Server error: {err}");
            Html(format!("{:#?}", err))
        }
    }
}

fn serve_page(context: &Context, template_name: &str, tera: &Tera) -> (StatusCode, Html<String>) {
    let index = tera.render(template_name, context);
    match index {
        Ok(content) => (StatusCode::OK, Html(content)),
        Err(err) => {
            tracing::error!("Server error: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("{:#?}", err)),
            )
        }
    }
}

fn get_embed(path: &str, asset: Option<rust_embed::EmbeddedFile>) -> impl IntoResponse {
    if let Some(file) = asset {
        (StatusCode::OK, Binary::new(file.data, path).into_response())
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
