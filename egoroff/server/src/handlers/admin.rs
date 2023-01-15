use super::*;

pub async fn serve_admin(State(page_context): State<Arc<PageContext>>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert(TITLE_KEY, "Админка");
    context.insert(CONFIG_KEY, &page_context.site_config);

    serve_page(&context, "admin.html", &page_context.tera)
}
