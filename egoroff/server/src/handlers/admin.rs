use super::{template::Admin, *};

/// Service administration interface main page
pub async fn serve() -> impl IntoResponse {
    Admin {
        title: "Админка",
        ..Default::default()
    }
    .into_response()
}
