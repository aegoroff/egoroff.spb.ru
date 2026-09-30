use kernel::{
    blog::Lookup,
    domain::{ApiResult, Archive, Post, PostsRequest, SmallPost},
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
) -> Result<Response, PageError> {
    serve_index(request, page_context, 1).await
}

pub async fn serve_index_not_default(
    Query(request): Query<BlogRequest>,
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(page): extract::Path<String>,
) -> Result<Response, PageError> {
    let page = page.parse().map_err(|e| {
        PageError::not_found().caused_by(anyhow::anyhow!("Invalid page '{page}': {e}"))
    })?;
    serve_index(request, page_context, page).await
}

async fn serve_index(
    request: BlogRequest,
    page_context: Arc<PageContext<'_>>,
    page: i32,
) -> Result<Response, PageError> {
    if page < 1 {
        return Err(PageError::not_found());
    }
    let section = find_section(&page_context, "blog")?;

    let req = PostsRequest {
        page: Some(page),
        tag: request.tag().map(ToOwned::to_owned),
        ..Default::default()
    };
    let poster = Poster::new(page_context.blog.page(req).await?, page);

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

    Ok(tpl.into_response())
}

pub async fn serve_document(
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(path): extract::Path<String>,
) -> Result<Response, PageError> {
    let id: i64 = strip_extension(&path).parse().map_err(|e| {
        PageError::not_found().caused_by(anyhow::anyhow!("Invalid post id '{path}': {e}"))
    })?;

    let article = match page_context.blog.article(id).await? {
        Lookup::Found(article) => article,
        Lookup::Moved(new_id) => return Ok(redirect_response(&format!("/blog/{new_id}.html"))),
        Lookup::NotFound => return Err(PageError::not_found()),
    };

    let uri = format!("{BLOG_PATH}{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);
    let keywords = article.post.keywords();
    Ok(BlogPost {
        html_class: "blog",
        title: &article.post.title,
        title_path: &title_path,
        keywords: &keywords,
        main_post: &article.post,
        content: article.html,
        meta_description: article.description,
        year: get_year(),
    }
    .into_response())
}

/// Just redirects to /blog/ page using 308 code
pub async fn redirect() -> impl IntoResponse {
    (
        StatusCode::PERMANENT_REDIRECT,
        Redirect::permanent("/blog/"),
    )
}

pub async fn serve_atom(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> Result<Content<String>, ApiError> {
    let posts = page_context.blog.recent(FEED_SIZE).await?;
    let xml = atom::from_small_posts(posts)?;
    Ok(Content(xml, "application/atom+xml; charset=utf-8"))
}

pub async fn serve_archive_api(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> JsonResult<Archive> {
    Ok(Json(page_context.blog.archive().await?))
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
) -> JsonResult<ApiResult<SmallPost>> {
    Ok(Json(page_context.blog.page(request).await?))
}

pub async fn serve_posts_admin_api(
    State(page_context): State<Arc<PageContext<'_>>>,
    Query(request): Query<PostsRequest>,
) -> JsonResult<ApiResult<Post>> {
    Ok(Json(page_context.blog.admin_page(request.page).await?))
}

pub async fn serve_post_create(
    State(page_context): State<Arc<PageContext<'_>>>,
    Json(post): Json<Post>,
) -> OperationResponse {
    page_context.blog.create(|id| Post { id, ..post }).await?;
    Ok(created())
}

pub async fn serve_post_update(
    State(page_context): State<Arc<PageContext<'_>>>,
    Json(post): Json<Post>,
) -> OperationResponse {
    page_context.blog.update(post).await?;
    Ok(updated())
}

pub async fn serve_post_delete(
    extract::Path(id): extract::Path<i64>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> OperationResponse {
    page_context.blog.delete(id).await?;
    Ok(updated())
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
