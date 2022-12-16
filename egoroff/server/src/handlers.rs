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
    domain::{PostsRequest, Storage},
    graph::SiteSection,
    sqlite::{Mode, Sqlite},
};
use oauth2::{CsrfToken, PkceCodeVerifier};
use rust_embed::RustEmbed;
use tera::{Context, Tera};

use crate::{
    atom,
    auth::{Authorizer, GoogleAuthorizer},
    body::Xml,
    domain::{AuthRequest, BlogRequest, Error, Navigation, PageContext, Poster, Uri},
    sitemap,
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
    context.insert("posts", &result.result);

    match apache_documents(&page_context.base_path) {
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

    match apache_documents(&page_context.base_path) {
        Ok(docs) => {
            context.insert("apache_docs", &docs);
            (
                StatusCode::OK,
                serve_page(&context, "portfolio/index.html", &page_context.tera),
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

pub async fn serve_portfolio_document(
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let mut context = Context::new();

    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("html_class", "");
    context.insert("ctx", "");
    context.insert("config", &page_context.site_config);

    let apache_documents = match apache_documents(&page_context.base_path) {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("{e:#?}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                make_500_page(&mut context, &page_context.tera),
            );
        }
    };

    let map: HashMap<&str, &crate::domain::Apache> = apache_documents
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();

    let doc = path.trim_end_matches(".html");

    let doc = match map.get(doc) {
        Some(item) => item,
        None => {
            return (
                StatusCode::NOT_FOUND,
                make_404_page(&mut context, &page_context.tera),
            )
        }
    };

    let uri = page_context.site_graph.full_path("portfolio");
    let uri = format!("{uri}/{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    context.insert(TITLE_KEY, &doc.title);
    context.insert("title_path", &title_path);
    context.insert("keywords", &doc.keywords);
    context.insert("meta_description", &doc.description);

    let asset = ApacheTemplates::get(&path);
    if let Some(file) = asset {
        let content = String::from_utf8_lossy(&file.data);
        context.insert("content", &content);
        (
            StatusCode::OK,
            serve_page(&context, "portfolio/apache.html", &page_context.tera),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            make_404_page(&mut context, &page_context.tera),
        )
    }
}

pub async fn serve_blog_default(
    axum::extract::Query(request): axum::extract::Query<BlogRequest>,
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    serve_blog_index(request, page_context, None)
}

pub async fn serve_blog_not_default_page(
    axum::extract::Query(request): axum::extract::Query<BlogRequest>,
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(page): extract::Path<String>,
) -> impl IntoResponse {
    serve_blog_index(request, page_context, Some(page))
}

fn serve_blog_index(
    request: BlogRequest,
    page_context: Arc<PageContext>,
    page: Option<String>,
) -> impl IntoResponse {
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
                return (
                    StatusCode::NOT_FOUND,
                    make_404_page(&mut context, &page_context.tera),
                );
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

    let poster = Poster::new(result, page);

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
    context.insert("poster", &poster);

    (
        StatusCode::OK,
        serve_page(&context, "blog/index.html", &page_context.tera),
    )
}

pub async fn serve_blog_page(
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
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
            return (
                StatusCode::NOT_FOUND,
                make_404_page(&mut context, &page_context.tera),
            );
        }
    };

    let storage = Sqlite::open(&page_context.storage_path, Mode::ReadOnly).unwrap();

    let post = match storage.get_post(id) {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Post ID '{id}' not found: {e:#?}");
            return (
                StatusCode::NOT_FOUND,
                make_404_page(&mut context, &page_context.tera),
            );
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
        Ok(c) => {
            context.insert("content", &c);
            (
                StatusCode::OK,
                serve_page(&context, "blog/post.html", &page_context.tera),
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

pub async fn serve_login(
    Extension(page_context): Extension<Arc<PageContext>>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    mut session: WritableSession,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("html_class", "");

    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);

    context.insert("ctx", "");
    context.insert("config", &page_context.site_config);

    let (authorize_url, csrf_state, pkce_code_verifier) =
        google_authorizer.generate_authorize_url();

    session.insert("csrf_state", csrf_state).unwrap();
    session
        .insert("pkce_code_verifier", pkce_code_verifier)
        .unwrap();

    context.insert(TITLE_KEY, "Авторизация");
    context.insert("google_signin_url", authorize_url.as_str());
    context.insert("github_signin_url", "");

    (
        StatusCode::OK,
        serve_page(&context, "signin.html", &page_context.tera),
    )
}

pub async fn google_oauth_callback(
    Query(query): Query<AuthRequest>,
    Extension(google_authorizer): Extension<Arc<GoogleAuthorizer>>,
    session: ReadableSession,
) -> impl IntoResponse {
    tracing::debug!("{query:#?}");
    match session.get::<CsrfToken>("csrf_state") {
        Some(original_csrf_state) => {
            if original_csrf_state.secret() == query.state.secret() {
                tracing::info!("authorized");
            } else {
                tracing::error!("unauthorized");
            }
        }
        None => tracing::error!("No state from session"),
    }
    match session.get::<PkceCodeVerifier>("pkce_code_verifier") {
        Some(pkce_code_verifier) => {
            let token = google_authorizer.exchange_code(query.code, pkce_code_verifier);
            match token {
                Ok(token) => {
                    tracing::info!("token: {token:#?}");
                }
                Err(e) => tracing::error!("token error: {e:#?}"),
            }
        }
        None => tracing::error!("No code verifier from session"),
    }

    drop(session);

    Redirect::to("/login")
}

pub async fn serve_atom(Extension(page_context): Extension<Arc<PageContext>>) -> impl IntoResponse {
    let req = PostsRequest {
        ..Default::default()
    };

    let result = archive::get_posts(&page_context.storage_path, 20, req);
    let xml = atom::from_small_posts(result.result).unwrap();

    Xml(xml)
}

pub async fn serve_sitemap(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let apache_documents = apache_documents(&page_context.base_path);

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

fn apache_documents(base_path: &Path) -> Result<Vec<crate::domain::Apache>> {
    let config_path = base_path.join("apache/config.json");
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
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
