use super::*;

/// Service administration interface main page
pub async fn serve(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(TITLE_KEY, "Админка");
    context.insert(CONFIG_KEY, &page_context.site_config);

    serve_page(&context, "admin.html", &page_context.tera)
}
