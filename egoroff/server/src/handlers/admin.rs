use super::*;

#[derive(Template, Default)]
#[template(path = "admin.html")]
struct Admin<'a> {
    html_class: &'a str,
    title: &'a str,
    title_path: &'a str,
    keywords: &'a str,
    meta_description: &'a str,
}

/// Service administration interface main page
pub async fn serve() -> impl IntoResponse {
    serve_page(Admin {
        title: "Админка",
        ..Default::default()
    })
}
