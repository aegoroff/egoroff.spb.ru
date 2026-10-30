use super::{Arc, IntoResponse, Json, PageContext, State, Storage, get_year, template::Admin};
use serde::Serialize;

/// Service administration interface main page
pub async fn serve() -> impl IntoResponse {
    Admin {
        title: "Админка",
        year: get_year(),
        ..Default::default()
    }
    .into_response()
}

#[derive(Serialize, Default)]
pub struct DashboardStats {
    pub posts: i32,
    pub downloads: i32,
    pub users: i32,
}

pub async fn serve_dashboard_api(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let posts_count = page_context.blog.count().await.unwrap_or(0);
    let (downloads_count, users_count) = page_context
        .storage
        .call(|storage| {
            Ok((
                storage.count_downloads().unwrap_or(0),
                storage.count_users().unwrap_or(0),
            ))
        })
        .await
        .unwrap_or((0, 0));

    Json(DashboardStats {
        posts: posts_count,
        downloads: downloads_count,
        users: users_count,
    })
}
