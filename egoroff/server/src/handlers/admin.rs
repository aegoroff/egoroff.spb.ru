use super::{template::Admin, *};

/// Service administration interface main page
pub async fn serve() -> impl IntoResponse {
    serve_page(Admin {
        title: "Админка",
        ..Default::default()
    })
}
