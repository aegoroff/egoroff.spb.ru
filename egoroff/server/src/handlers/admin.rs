use super::*;

pub async fn serve_admin(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(HTML_CLASS_KEY, "");
    context.insert(TITLE_KEY, "Админка");
    context.insert(CONFIG_KEY, &page_context.site_config);

    serve_page(&context, "admin.html", &page_context.tera)
}