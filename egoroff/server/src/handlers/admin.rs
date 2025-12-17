use super::{template::Admin, *};

/// Service administration interface main page
pub async fn serve() -> impl IntoResponse {
    Admin {
        title: "Админка",
        year: get_year(),
        ..Default::default()
    }
    .into_response()
}
