use super::*;

pub async fn serve_admin(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("html_class", "");
    context.insert(TITLE_KEY, "Админка");
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", "");
    context.insert("meta_description", "");
    context.insert("config", &page_context.site_config);

    serve_page(&context, "admin.html", &page_context.tera)
}