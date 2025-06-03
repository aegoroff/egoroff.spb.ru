use kernel::{
    converter::html2text,
    domain::{ApiResult, Post, SmallPost},
};

use crate::body::{Content, Redirect};

use super::{
    template::{BlogIndex, BlogPost},
    *,
};

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

static REPLACES_MAP: std::sync::LazyLock<HashMap<&'static str, &'static str>> =
    std::sync::LazyLock::new(|| OPINIONS_REMAP.iter().map(|(k, v)| (*k, *v)).collect());

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
    let page = if let Some(page) = page {
        match page.parse() {
            Ok(item) => item,
            Err(e) => {
                tracing::error!("Invalid page: {e:#?}");
                return make_404_page();
            }
        }
    } else {
        1
    };

    let Some(section) = page_context.site_graph.get_section("blog") else {
        return make_500_page();
    };

    let req = PostsRequest {
        page: Some(page),
        ..Default::default()
    };

    let storage = page_context.storage.lock().await;
    let result = archive::get_small_posts(storage, PAGE_SIZE, Some(req));

    let api_result = match result {
        Ok(ar) => ar,
        Err(e) => {
            tracing::error!("Get posts error: {e:#?}");
            return make_500_page();
        }
    };

    let poster = Poster::new(api_result, page);

    let mut tpl = BlogIndex {
        html_class: "blog",
        title: &section.title,
        title_path: "",
        keywords: get_keywords(section),
        meta_description: "",
        flashed_messages: vec![],
        poster: &poster,
        request: &request,
    };

    let title = format!("{page}-я страница");
    let description = format!("{} {title}", section.descr);
    let title_path = if page == 1 {
        page_context.site_graph.make_title_path(BLOG_PATH)
    } else {
        tpl.title = &title;
        tpl.meta_description = &description;
        page_context
            .site_graph
            .make_title_path(&format!("{BLOG_PATH}{page}"))
    };
    tpl.title_path = &title_path;

    serve_page(tpl)
}

pub async fn serve_document(
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let doc = strip_extension(&path);

    let id: i64 = match doc.parse() {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Invalid post id: {e:#?}. Expected number but was {doc}");
            return make_404_page();
        }
    };

    let storage = page_context.storage.lock().await;

    if let Ok(id) = storage.get_new_post_id(id) {
        let new_path = format!("/blog/{id}.html");
        return redirect_response(&new_path);
    }

    let post = match storage.get_post(id) {
        Ok(item) => item,
        Err(e) => {
            tracing::error!("Post ID '{id}' not found: {e:#?}");
            return make_404_page();
        }
    };
    drop(storage);
    let uri = format!("{BLOG_PATH}{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    let content = if post.markdown {
        markdown2html(&post.text)
    } else if post.text.starts_with("<?xml version=\"1.0\"?>") {
        xml2html(&post.text)
    } else {
        Ok(post.text.clone())
    };

    match content {
        Ok(c) => {
            let meta_description = if c.is_empty() {
                post.title.clone()
            } else {
                let descr = if post.markdown {
                    markdown2html(&post.short_text).unwrap_or_default()
                } else {
                    post.short_text.clone()
                };
                if descr.is_empty() {
                    descr
                } else if let Ok(txt) = html2text(&descr) {
                    txt
                } else {
                    descr
                }
            };

            let keywords = post.keywords();
            serve_page(BlogPost {
                html_class: "blog",
                title: &post.title,
                title_path: &title_path,
                keywords: &keywords,
                flashed_messages: vec![],
                main_post: &post,
                content: &c,
                meta_description,
            })
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            make_500_page()
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

pub async fn serve_archive_api(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
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
        (status = 200, description = "Get posts successfully", body = ApiResult<SmallPost>),
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
