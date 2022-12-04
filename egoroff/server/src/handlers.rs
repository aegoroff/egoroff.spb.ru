use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use axum::{
    body::{Bytes, Empty, Full},
    extract,
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse},
    Extension, Json,
};
use axum_extra::either::Either;
use kernel::graph::{SiteGraph, SiteSection};
use rust_embed::RustEmbed;
use tera::{Context, Tera, Value};

use crate::domain::{Config, Navigation, Poster, Uri};

#[cfg(debug_assertions)]
const MODE: &str = "debug";

#[cfg(not(debug_assertions))]
const MODE: &str = "release";

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
    Extension(base_path): Extension<PathBuf>,
    Extension(site_graph): Extension<SiteGraph>,
    Extension(site_config): Extension<Config>,
) -> impl IntoResponse {
    let section = site_graph.get_section("/").unwrap();
    let mut context = Context::new();
    context.insert("html_class", "welcome");
    context.insert("title", "egoroff.spb.ru");
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("config", &site_config);
    context.insert("ctx", "");
    let apache_documents = apache_documents(&base_path);
    context.insert("apache_docs", &apache_documents);

    serve_page(&context, "welcome.html", base_path, site_graph)
}

pub async fn serve_portfolio(
    Extension(base_path): Extension<PathBuf>,
    Extension(site_graph): Extension<SiteGraph>,
    Extension(site_config): Extension<Config>,
) -> impl IntoResponse {
    let section = site_graph.get_section("portfolio").unwrap();

    let mut context = Context::new();
    context.insert("html_class", "portfolio");
    context.insert("title", &section.title);
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("config", &site_config);
    context.insert("ctx", "");
    let apache_documents = apache_documents(&base_path);
    context.insert("apache_docs", &apache_documents);

    serve_page(&context, "portfolio/index.html", base_path, site_graph)
}

pub async fn serve_portfolio_document(
    Extension(base_path): Extension<PathBuf>,
    Extension(site_graph): Extension<SiteGraph>,
    Extension(site_config): Extension<Config>,
    extract::Path(path): extract::Path<String>,
) -> Either<Html<String>, (StatusCode, Empty<Bytes>)> {
    let asset = ApacheTemplates::get(&path);
    let apache_documents = apache_documents(&base_path);
    let map: HashMap<&str, &crate::domain::Apache> = apache_documents
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();

    let doc = path.trim_end_matches(".html");

    let doc = match map.get(doc) {
        Some(item) => item,
        None => return Either::E2((StatusCode::NOT_FOUND, Empty::new())),
    };

    let mut context = Context::new();
    context.insert("title", &doc.title);
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &doc.keywords);
    context.insert("meta_description", &doc.description);
    context.insert("html_class", "");
    context.insert("ctx", "");
    context.insert("config", &site_config);
    if let Some(file) = asset {
        let content = String::from_utf8_lossy(&file.data);
        context.insert("content", &content);
        Either::E1(serve_page(
            &context,
            "portfolio/apache.html",
            base_path,
            site_graph,
        ))
    } else {
        Either::E2((StatusCode::NOT_FOUND, Empty::new()))
    }
}

pub async fn serve_blog(
    Extension(base_path): Extension<PathBuf>,
    Extension(site_graph): Extension<SiteGraph>,
    Extension(site_config): Extension<Config>,
) -> impl IntoResponse {
    let section = site_graph.get_section("blog").unwrap();

    let mut context = Context::new();
    context.insert("html_class", "blog");
    context.insert("title", &section.title);
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("ctx", "");
    context.insert("config", &site_config);
    let poster = Poster {
        small_posts: vec![],
    };
    context.insert("poster", &poster);

    serve_page(&context, "blog/index.html", base_path, site_graph)
}

pub async fn serve_search(
    Extension(base_path): Extension<PathBuf>,
    Extension(site_graph): Extension<SiteGraph>,
    Extension(site_config): Extension<Config>,
) -> impl IntoResponse {
    let section = site_graph.get_section("search").unwrap();

    let mut context = Context::new();
    context.insert("html_class", "search");
    context.insert("title", &section.title);
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("ctx", "");
    context.insert("config", &site_config);

    serve_page(&context, "search.html", base_path, site_graph)
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
    Extension(site_graph): Extension<SiteGraph>,
) -> impl IntoResponse {
    let q = query.uri;

    let (breadcrumbs, current) = site_graph.breadcrumbs(&q);

    let breadcrumbs = if q != "/" { Some(breadcrumbs) } else { None };

    match site_graph.get_section("/") {
        Some(r) => Json(Navigation {
            sections: activate_section(r.children, &current),
            breadcrumbs,
        }),
        None => Json(Navigation {
            ..Default::default()
        }),
    }
}

fn serve_page(
    context: &Context,
    template_name: &str,
    base_path: PathBuf,
    site_graph: SiteGraph,
) -> Html<String> {
    let templates_path = base_path.join("static/dist/**/*.html");
    let templates_path = templates_path.to_str().unwrap();

    let mut tera = match Tera::new(templates_path) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Server error: {e}");
            return Html(format!("Parsing error(s): {:#?}", e));
        }
    };
    tera.register_function(
        "path_for",
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("id") {
                Some(val) => match tera::from_value::<String>(val.clone()) {
                    Ok(v) => Ok(tera::to_value(site_graph.full_path(&v)).unwrap()),
                    Err(_) => Err("oops".into()),
                },
                None => Err("oops".into()),
            }
        },
    );

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
