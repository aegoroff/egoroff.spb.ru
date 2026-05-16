use super::{template::Admin, *};
use kernel::domain::{ApiResult, PostsRequest, User};
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
    pub posts_count: i32,
    pub downloads_count: i32,
    pub users_count: i32,
}

pub async fn serve_dashboard_api(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let storage = page_context.storage.lock().await;
    
    let posts_count = storage.count_posts(PostsRequest::default()).unwrap_or(0);
    let downloads_count = storage.count_downloads().unwrap_or(0);
    let users_count = storage.count_users().unwrap_or(0);
    
    success_response(Json(DashboardStats {
        posts_count,
        downloads_count,
        users_count,
    }))
}

pub async fn serve_users_api(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let storage = page_context.storage.lock().await;

    let users = match storage.get_users() {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to get users: {e:#?}");
            return make_json_response::<ApiResult<User>>(Err(anyhow::anyhow!(e)));
        }
    };
    let users_count = users.len() as i32;

    let result = ApiResult {
        result: users,
        pages: 1,
        page: 1,
        count: users_count,
        status: "success",
    };

    make_json_response(Ok(result))
}
