use kernel::domain::Post;

use crate::{body::Content, domain::OperationResult};

use super::*;

const PAGE_SIZE: i32 = 20;

const OPINIONS_REMAP: &[(&str, &str)] = &[
    ("1", "25002"),
    ("4", "31001"),
    ("8", "6003"),
    ("11", "30001"),
    ("13", "3006"),
    ("18", "29001"),
    ("21", "9002"),
    ("22", "2004"),
    ("24", "25003"),
    ("25", "22002"),
    ("26", "27002"),
    ("27", "27001"),
    ("28", "14004"),
    ("29", "8003"),
    ("30", "6004"),
];

lazy_static::lazy_static! {
    static ref REPLACES_MAP: HashMap<&'static str, &'static str> = OPINIONS_REMAP.iter().map(|(k, v)| (*k, *v)).collect();
}

pub async fn serve_index_default(
    axum::extract::Query(request): axum::extract::Query<BlogRequest>,
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    serve_index(request, page_context, None)
}

pub async fn serve_index_not_default(
    axum::extract::Query(request): axum::extract::Query<BlogRequest>,
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(page): extract::Path<String>,
) -> impl IntoResponse {
    serve_index(request, page_context, Some(page))
}

fn serve_index(
    request: BlogRequest,
    page_context: Arc<PageContext>,
    page: Option<String>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(HTML_CLASS_KEY, "blog");
    context.insert(CONFIG_KEY, &page_context.site_config);

    let page = if let Some(page) = page {
        match page.parse() {
            Ok(item) => item,
            Err(e) => {
                tracing::error!("Invalid page: {e:#?}");
                return make_404_page(&mut context, &page_context.tera);
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

    let result = archive::get_small_posts(&page_context.storage_path, PAGE_SIZE, req);

    let poster = Poster::new(result, page);

    let mut title = section.title;
    let mut uri = page_context.site_graph.full_path("blog");
    if page != 1 {
        title = format!("{page}-я страница");
        uri = format!("{uri}{page}")
    }

    let title_path = page_context.site_graph.make_title_path(&uri);

    context.insert(TITLE_KEY, &title);
    context.insert(KEYWORDS_KEY, &section.keywords);
    context.insert(META_KEY, &section.descr);
    context.insert("request", &request);
    context.insert(TITLE_PATH_KEY, &title_path);
    context.insert("poster", &poster);

    serve_page(&context, "blog/index.html", &page_context.tera)
}

pub async fn serve_document(
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(HTML_CLASS_KEY, "blog");
    context.insert(CONFIG_KEY, &page_context.site_config);

    let doc = path.trim_end_matches(".html");

    let id: i64 = match doc.parse() {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Invalid post id: {e:#?}. Expected number but was {doc}");
            return make_404_page(&mut context, &page_context.tera);
        }
    };

    let storage = Sqlite::open(&page_context.storage_path, Mode::ReadOnly).unwrap();

    let post = match storage.get_post(id) {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Post ID '{id}' not found: {e:#?}");
            return make_404_page(&mut context, &page_context.tera);
        }
    };
    let uri = page_context.site_graph.full_path("blog");
    let uri = format!("{uri}/{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    context.insert(TITLE_KEY, &post.title);
    context.insert(TITLE_PATH_KEY, &title_path);

    let keywords = post.keywords();

    context.insert(KEYWORDS_KEY, &keywords);
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
            serve_page(&context, "blog/post.html", &page_context.tera)
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            make_500_page(&mut context, &page_context.tera)
        }
    }
}

pub async fn serve_atom(Extension(page_context): Extension<Arc<PageContext>>) -> impl IntoResponse {
    let req = PostsRequest {
        ..Default::default()
    };

    let result = archive::get_small_posts(&page_context.storage_path, 20, req);
    let xml = atom::from_small_posts(result.result).unwrap();

    Content(xml, "application/atom+xml; charset=utf-8")
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
    let result = archive::get_small_posts(&page_context.storage_path, PAGE_SIZE, request);
    Json(result)
}

pub async fn service_posts_admin_api(
    Extension(page_context): Extension<Arc<PageContext>>,
    axum::extract::Query(request): axum::extract::Query<PostsRequest>,
) -> impl IntoResponse {
    let result = archive::get_posts(&page_context.storage_path, 10, request);
    Json(result)
}

pub async fn service_post_update(
    Extension(page_context): Extension<Arc<PageContext>>,
    Json(post): Json<Post>,
) -> impl IntoResponse {
    let mut storage = Sqlite::open(&page_context.storage_path, Mode::ReadWrite).unwrap();
    match storage.upsert_post(post) {
        Ok(_) => Json(OperationResult {
            result: "success".to_owned(),
        }),
        Err(e) => Json(OperationResult {
            result: format!("{e}"),
        }),
    }
}

pub async fn redirect_to_real_document(
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let id = path
        .strip_suffix(".html")
        .unwrap_or_else(|| path.strip_suffix(".htm").unwrap_or_default());

    if id.is_empty() {
        Redirect::permanent("/blog/")
    } else if REPLACES_MAP.contains_key(id) {
        let new_page = REPLACES_MAP.get(id).unwrap();
        let new_path = format!("/blog/{new_page}.html");
        Redirect::permanent(&new_path)
    } else {
        Redirect::permanent("/blog/")
    }
}
