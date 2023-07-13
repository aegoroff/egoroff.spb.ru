use std::fmt::Display;

use kernel::{converter::html2text, domain::Post};

use crate::{
    body::{Content, Redirect},
    domain::OperationResult,
};

use super::*;

const PAGE_SIZE: i32 = 20;

const OPINIONS_REMAP: &[(&str, &str)] = &[
    ("1", "1"),
    ("4", "6"),
    ("8", "11"),
    ("11", "14"),
    ("13", "18"),
    ("18", "27"),
    ("21", "28"),
    ("22", "29"),
    ("24", "33"),
    ("25", "35"),
    ("26", "37"),
    ("27", "36"),
    ("28", "42"),
    ("29", "43"),
    ("30", "44"),
];

const BLOG_PATH: &str = "/blog/";

lazy_static::lazy_static! {
    static ref REPLACES_MAP: HashMap<&'static str, &'static str> = OPINIONS_REMAP.iter().map(|(k, v)| (*k, *v)).collect();
}

pub async fn serve_index_default(
    Query(request): Query<BlogRequest>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    serve_index(request, page_context, None).await
}

pub async fn serve_index_not_default(
    Query(request): Query<BlogRequest>,
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(page): extract::Path<String>,
) -> impl IntoResponse {
    serve_index(request, page_context, Some(page)).await
}

async fn serve_index(
    request: BlogRequest,
    page_context: Arc<PageContext<'_>>,
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

    let Some(section) = page_context.site_graph.get_section("blog") else { return make_500_page(&mut context, &page_context.tera) };

    let req = PostsRequest {
        page: Some(page),
        ..Default::default()
    };

    let storage = page_context.storage.lock().await;
    let result = archive::get_small_posts(storage, PAGE_SIZE, Some(req));

    let posts = match result {
        Ok(ar) => ar,
        Err(e) => {
            tracing::error!("Get posts error: {e:#?}");
            return make_500_page(&mut context, &page_context.tera);
        }
    };

    let poster = Poster::new(posts, page);

    let title_path = if page == 1 {
        context.insert(TITLE_KEY, &section.title);
        context.insert(META_DESCR_KEY, &section.descr);
        page_context.site_graph.make_title_path(BLOG_PATH)
    } else {
        let title = format!("{page}-я страница");
        let description = format!("{} {title}", section.descr);
        context.insert(TITLE_KEY, &title);
        context.insert(META_DESCR_KEY, &description);
        page_context
            .site_graph
            .make_title_path(&format!("{BLOG_PATH}{page}"))
    };

    context.insert(KEYWORDS_KEY, &section.keywords);
    context.insert("request", &request);
    context.insert(TITLE_PATH_KEY, &title_path);
    context.insert("poster", &poster);

    serve_page(&context, "blog/index.html", &page_context.tera)
}

pub async fn serve_document(
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(HTML_CLASS_KEY, "blog");
    context.insert(CONFIG_KEY, &page_context.site_config);

    let doc = strip_extension(&path);

    let id: i64 = match doc.parse() {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Invalid post id: {e:#?}. Expected number but was {doc}");
            return make_404_page(&mut context, &page_context.tera);
        }
    };

    let storage = page_context.storage.lock().await;

    if let Ok(id) = storage.get_new_post_id(id) {
        let new_path = format!("/blog/{id}.html");
        return (
            StatusCode::PERMANENT_REDIRECT,
            Redirect::permanent(&new_path).into_response(),
        );
    }

    let post = match storage.get_post(id) {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Post ID '{id}' not found: {e:#?}");
            return make_404_page(&mut context, &page_context.tera);
        }
    };
    drop(storage);
    let uri = format!("{BLOG_PATH}{path}");
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
            if !c.is_empty() {
                let descr = if post.markdown {
                    markdown2html(&post.short_text).unwrap_or_default()
                } else {
                    post.short_text
                };
                if !descr.is_empty() {
                    if let Ok(txt) = html2text(&descr) {
                        context.insert(META_DESCR_KEY, &txt);
                    }
                }
            }

            context.insert("content", &c);
            serve_page(&context, "blog/post.html", &page_context.tera)
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            make_500_page(&mut context, &page_context.tera)
        }
    }
}

/// Just redirects to /blog/ page using 308 code
pub async fn redirect() -> impl IntoResponse {
    (
        StatusCode::PERMANENT_REDIRECT,
        Redirect::permanent("/blog/"),
    )
}

pub async fn serve_atom(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let storage = page_context.storage.lock().await;
    let result = archive::get_small_posts(storage, 20, None);

    match result {
        Ok(r) => match atom::from_small_posts(r.result) {
            Ok(xml) => success_response(Content(xml, "application/atom+xml; charset=utf-8")),
            Err(e) => {
                tracing::error!("Convert atom posts error: {e:#?}");
                internal_server_error_response(Content(e.to_string(), "text/plain; charset=utf-8"))
            }
        },
        Err(e) => {
            tracing::error!("Get posts error: {e:#?}");
            internal_server_error_response(Content(e.to_string(), "text/plain; charset=utf-8"))
        }
    }
}

pub async fn serve_archive_api(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let storage = page_context.storage.lock().await;
    let result = archive::archive(storage);
    make_json_response(result)
}

/// Gets small blog posts without full test (only short description and metadata) using various queries.
#[utoipa::path(
    get,
    path = "/api/v2/blog/posts/",
    params(
        PostsRequest
    ),
    tag = "blog",
    responses(
        (status = 200, description = "Get posts successfully", body = SmallPosts),
    ),
)]
pub async fn serve_posts_api(
    State(page_context): State<Arc<PageContext<'_>>>,
    Query(request): Query<PostsRequest>,
) -> impl IntoResponse {
    let storage = page_context.storage.lock().await;
    let result = archive::get_small_posts(storage, PAGE_SIZE, Some(request));
    make_json_response(result)
}

pub async fn serve_posts_admin_api(
    State(page_context): State<Arc<PageContext<'_>>>,
    Query(request): Query<PostsRequest>,
) -> impl IntoResponse {
    let storage = page_context.storage.lock().await;
    let result = archive::get_posts(&storage, 10, request);
    make_json_response(result)
}

pub async fn serve_post_update(
    State(page_context): State<Arc<PageContext<'_>>>,
    Json(post): Json<Post>,
) -> impl IntoResponse {
    let mut storage = page_context.storage.lock().await;
    let result = storage.upsert_post(post);
    updated_response(result)
}

pub async fn serve_post_delete(
    extract::Path(id): extract::Path<i64>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let mut storage = page_context.storage.lock().await;
    let result = storage.delete_post(id);
    updated_response(result)
}

pub async fn redirect_to_real_document(
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let id = strip_extension(&path);

    match REPLACES_MAP.get(id) {
        Some(new_page) => {
            let new_path = format!("/blog/{new_page}.html");
            Redirect::permanent(&new_path)
        }
        None => Redirect::permanent("/blog/"),
    }
}

fn updated_response<T, E: Display>(result: Result<T, E>) -> impl IntoResponse {
    if let Err(e) = result {
        let error = format!("{e}");
        internal_server_error_response(Json(OperationResult { result: &error }))
    } else {
        success_response(Json(OperationResult { result: "success" }))
    }
}

fn strip_extension(path: &str) -> &str {
    path.strip_suffix(".html")
        .unwrap_or_else(|| path.strip_suffix(".htm").unwrap_or(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("1.html", "1")]
    #[case("1.htm", "1")]
    #[case("100000.html", "100000")]
    #[case("100000", "100000")]
    #[case("", "")]
    #[case("a", "a")]
    #[trace]
    fn strip_extension_tests(#[case] test_data: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = strip_extension(test_data);

        // assert
        assert_eq!(expected, actual)
    }
}
