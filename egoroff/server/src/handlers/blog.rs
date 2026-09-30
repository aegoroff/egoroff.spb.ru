use kernel::{
    blog::Lookup,
    domain::{ApiResult, Post, PostsRequest, SmallPost},
};

use crate::body::Content;
use axum::response::Redirect;

use super::{
    template::{BlogIndex, BlogPost},
    *,
};

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

/// Posts in the Atom feed.
const FEED_SIZE: i32 = 20;

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
                return not_found_page();
            }
        }
    } else {
        1
    };
    if page < 1 {
        return not_found_page();
    }

    let Some(section) = page_context.site_graph.get_section("blog") else {
        return internal_server_error_page();
    };

    let req = PostsRequest {
        page: Some(page),
        tag: request.tag.clone(),
        ..Default::default()
    };

    let api_result = match page_context.blog.page(req).await {
        Ok(ar) => ar,
        Err(e) => {
            tracing::error!("Get posts error: {e:#?}");
            return internal_server_error_page();
        }
    };

    let poster = Poster::new(api_result, page);

    let mut tpl = BlogIndex {
        html_class: "blog",
        title: &section.title,
        title_path: "",
        keywords: get_keywords(section),
        meta_description: section.descr.as_str(),
        poster: &poster,
        request: &request,
        year: get_year(),
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

    tpl.into_response()
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
            return not_found_page();
        }
    };

    let article = match page_context.blog.article(id).await {
        Ok(Lookup::Found(article)) => article,
        Ok(Lookup::Moved(new_id)) => return redirect_response(&format!("/blog/{new_id}.html")),
        Ok(Lookup::NotFound) => return not_found_page(),
        Err(e) => {
            tracing::error!("Post ID '{id}' read error: {e:#?}");
            return internal_server_error_page();
        }
    };

    let uri = format!("{BLOG_PATH}{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);
    let keywords = article.post.keywords();
    BlogPost {
        html_class: "blog",
        title: &article.post.title,
        title_path: &title_path,
        keywords: &keywords,
        main_post: &article.post,
        content: article.html,
        meta_description: article.description,
        year: get_year(),
    }
    .into_response()
}

/// Just redirects to /blog/ page using 308 code
pub async fn redirect() -> impl IntoResponse {
    (
        StatusCode::PERMANENT_REDIRECT,
        Redirect::permanent("/blog/"),
    )
}

pub async fn serve_atom(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    match page_context.blog.recent(FEED_SIZE).await {
        Ok(posts) => match atom::from_small_posts(posts) {
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
    make_json_response(page_context.blog.archive().await)
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
    make_json_response(page_context.blog.page(request).await)
}

pub async fn serve_posts_admin_api(
    State(page_context): State<Arc<PageContext<'_>>>,
    Query(request): Query<PostsRequest>,
) -> impl IntoResponse {
    make_json_response(page_context.blog.admin_page(request.page).await)
}

pub async fn serve_post_create(
    State(page_context): State<Arc<PageContext<'_>>>,
    Json(post): Json<Post>,
) -> impl IntoResponse {
    created_response(page_context.blog.create(|id| Post { id, ..post }).await)
}

pub async fn serve_post_update(
    State(page_context): State<Arc<PageContext<'_>>>,
    Json(post): Json<Post>,
) -> impl IntoResponse {
    updated_response(page_context.blog.update(post).await)
}

pub async fn serve_post_delete(
    extract::Path(id): extract::Path<i64>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    updated_response(page_context.blog.delete(id).await)
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
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
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
