use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    body::{Empty, Full},
    extract,
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse},
    Extension, Json,
};
use axum_extra::either::{Either, Either3};
use kernel::{
    archive,
    converter::{markdown2html, xml2html},
    domain::{PostsRequest, Storage},
    graph::SiteSection,
    sqlite::{Mode, Sqlite},
};
use rust_embed::RustEmbed;
use tera::{Context, Tera};

use crate::{
    atom::{self, Xml},
    domain::{BlogRequest, Error, Navigation, PageContext, Poster, Uri},
};

const PAGE_SIZE: i32 = 20;

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

#[derive(RustEmbed)]
#[folder = "../../templates/apache"]
struct ApacheTemplates;

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
    let apache_documents = apache_documents(&page_context.base_path);
    context.insert("apache_docs", &apache_documents);
    context.insert("posts", &result.result);

    serve_page(&context, "welcome.html", &page_context.tera)
}

pub async fn serve_portfolio(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let section = page_context.site_graph.get_section("portfolio").unwrap();

    let uri = page_context.site_graph.full_path("portfolio");
    let title_path = page_context.site_graph.make_title_path(&uri);

    let mut context = Context::new();
    context.insert("html_class", "portfolio");
    context.insert(TITLE_KEY, &section.title);
    context.insert("title_path", &title_path);
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("config", &page_context.site_config);
    context.insert("ctx", "");
    let apache_documents = apache_documents(&page_context.base_path);
    context.insert("apache_docs", &apache_documents);

    serve_page(&context, "portfolio/index.html", &page_context.tera)
}

pub async fn serve_portfolio_document(
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(path): extract::Path<String>,
) -> Either<Html<String>, (StatusCode, Html<String>)> {
    let asset = ApacheTemplates::get(&path);
    let apache_documents = apache_documents(&page_context.base_path);
    let map: HashMap<&str, &crate::domain::Apache> = apache_documents
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();

    let mut context = Context::new();

    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("html_class", "");
    context.insert("ctx", "");
    context.insert("config", &page_context.site_config);

    let doc = path.trim_end_matches(".html");

    let doc = match map.get(doc) {
        Some(item) => item,
        None => {
            return Either::E2((
                StatusCode::NOT_FOUND,
                make_404_page(&mut context, &page_context.tera),
            ))
        }
    };

    let uri = page_context.site_graph.full_path("portfolio");
    let uri = format!("{uri}/{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    context.insert(TITLE_KEY, &doc.title);
    context.insert("title_path", &title_path);
    context.insert("keywords", &doc.keywords);
    context.insert("meta_description", &doc.description);

    if let Some(file) = asset {
        let content = String::from_utf8_lossy(&file.data);
        context.insert("content", &content);
        Either::E1(serve_page(
            &context,
            "portfolio/apache.html",
            &page_context.tera,
        ))
    } else {
        Either::E2((
            StatusCode::NOT_FOUND,
            make_404_page(&mut context, &page_context.tera),
        ))
    }
}

pub async fn serve_blog_default(
    axum::extract::Query(request): axum::extract::Query<BlogRequest>,
    Extension(page_context): Extension<Arc<PageContext>>,
) -> Either<Html<String>, (StatusCode, Html<String>)> {
    serve_blog_index(request, page_context, None)
}

pub async fn serve_blog_not_default_page(
    axum::extract::Query(request): axum::extract::Query<BlogRequest>,
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(page): extract::Path<String>,
) -> Either<Html<String>, (StatusCode, Html<String>)> {
    serve_blog_index(request, page_context, Some(page))
}

fn serve_blog_index(
    request: BlogRequest,
    page_context: Arc<PageContext>,
    page: Option<String>,
) -> Either<Html<String>, (StatusCode, Html<String>)> {
    let mut context = Context::new();
    context.insert("html_class", "blog");
    context.insert("gin_mode", MODE);
    context.insert("ctx", "");
    context.insert("config", &page_context.site_config);
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);

    let page = if let Some(page) = page {
        match page.parse() {
            Ok(item) => item,
            Err(e) => {
                tracing::error!("Invalid page: {e:#?}");
                return Either::E2((
                    StatusCode::NOT_FOUND,
                    make_404_page(&mut context, &page_context.tera),
                ));
            }
        }
    } else {
        1
    };

    let section = page_context.site_graph.get_section("blog").unwrap();
    let req = PostsRequest {
        page: Some(page),
        ..Default::default()
    };

    let result = archive::get_posts(&page_context.storage_path, PAGE_SIZE, req);
    let pages_count = result.pages;

    let pages: Vec<i32> = (1..=pages_count).collect();

    let mut title = section.title;
    let mut uri = page_context.site_graph.full_path("blog");
    if page != 1 {
        title = format!("{page}-я страница");
        uri = format!("{uri}{page}")
    }

    let title_path = page_context.site_graph.make_title_path(&uri);

    context.insert(TITLE_KEY, &title);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("request", &request);
    context.insert("title_path", &title_path);
    let prev_page = if page == 1 { 1 } else { page - 1 };
    let next_page = if page == pages_count {
        pages_count
    } else {
        page + 1
    };
    let poster = Poster {
        small_posts: result.result,
        has_pages: pages_count > 0,
        has_prev: page > 1,
        has_next: page < pages_count,
        page,
        prev_page,
        next_page,
        pages,
    };
    context.insert("poster", &poster);

    Either::E1(serve_page(&context, "blog/index.html", &page_context.tera))
}

pub async fn serve_blog_page(
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(path): extract::Path<String>,
) -> Either3<Html<String>, Xml<String>, (StatusCode, Html<String>)> {
    if path == "recent.atom" {
        return Either3::E2(make_atom_content(page_context));
    }

    let mut context = Context::new();
    context.insert("html_class", "blog");
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("ctx", "");
    context.insert("config", &page_context.site_config);

    let doc = path.trim_end_matches(".html");

    let id: i64 = match doc.parse() {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Invalid post id: {e:#?}. Expected number but was {doc}");
            return Either3::E3((
                StatusCode::NOT_FOUND,
                make_404_page(&mut context, &page_context.tera),
            ));
        }
    };

    let storage = Sqlite::open(&page_context.storage_path, Mode::ReadOnly).unwrap();

    let post = match storage.get_post(id) {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Post ID '{id}' not found: {e:#?}");
            return Either3::E3((
                StatusCode::NOT_FOUND,
                make_404_page(&mut context, &page_context.tera),
            ));
        }
    };
    let uri = page_context.site_graph.full_path("blog");
    let uri = format!("{uri}/{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    context.insert(TITLE_KEY, &post.title);
    context.insert("title_path", &title_path);

    let keywords = post.tags.join(",");

    context.insert("keywords", &keywords);
    context.insert("main_post", &post);

    let content = if post.markdown {
        markdown2html(&post.text)
    } else if post.text.starts_with("<?xml version=\"1.0\"?>") {
        xml2html(&post.text)
    } else {
        Ok(post.text)
    };

    match content {
        Ok(c) => context.insert("content", &c),
        Err(e) => {
            tracing::error!("{e:#?}");
            return Either3::E3((
                StatusCode::INTERNAL_SERVER_ERROR,
                make_500_page(&mut context, &page_context.tera),
            ));
        }
    }

    Either3::E1(serve_page(&context, "blog/post.html", &page_context.tera))
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

pub async fn serve_atom(Extension(page_context): Extension<Arc<PageContext>>) -> impl IntoResponse {
    make_atom_content(page_context)
}

fn make_atom_content(page_context: Arc<PageContext>) -> Xml<String> {
    let req = PostsRequest {
        ..Default::default()
    };

    let result = archive::get_posts(&page_context.storage_path, 20, req);
    let xml = atom::from_small_posts(result.result).unwrap();

    Xml(xml)
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

pub async fn serve_archive_api(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    Json(archive::archive(&page_context.storage_path))
}

pub async fn service_posts_api(
    Extension(page_context): Extension<Arc<PageContext>>,
    axum::extract::Query(request): axum::extract::Query<PostsRequest>,
) -> impl IntoResponse {
    let result = archive::get_posts(&page_context.storage_path, PAGE_SIZE, request);
    Json(result)
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

fn apache_documents(base_path: &Path) -> Vec<crate::domain::Apache> {
    let config_path = base_path.join("apache/config.json");
    let file = File::open(config_path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
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
