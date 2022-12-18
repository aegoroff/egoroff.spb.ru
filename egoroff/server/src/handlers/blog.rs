use super::*;

const PAGE_SIZE: i32 = 20;

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

pub async fn serve_atom(Extension(page_context): Extension<Arc<PageContext>>) -> impl IntoResponse {
    let req = PostsRequest {
        ..Default::default()
    };

    let result = archive::get_posts(&page_context.storage_path, 20, req);
    let xml = atom::from_small_posts(result.result).unwrap();

    Xml(xml)
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
