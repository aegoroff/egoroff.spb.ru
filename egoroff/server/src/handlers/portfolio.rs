use std::time::Duration;

use anyhow::Context;
use kernel::domain::{ApiResult, Download, DownloadsRequest};
use serde::Deserialize;

use crate::{
    body::Redirect,
    domain::{Downloadable, FilesContainer},
};

use super::{
    template::{ApacheDocument, Portfolio},
    *,
};

#[derive(RustEmbed)]
#[folder = "../../templates/apache"]
struct ApacheTemplates;

const PORTFOLIO_PATH: &str = "/portfolio/";
const DOWNLOADS_WAIT_TIMEOUT_SECONDS: u64 = 5;

#[derive(Deserialize, Default)]
pub struct StoredFile {
    pub id: i64,
    pub path: String,
    pub blake3_hash: String,
    pub size: u64,
}

pub async fn serve_index(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let Some(section) = page_context.site_graph.get_section("portfolio") else {
        return internal_server_error_page();
    };

    let title_path = page_context.site_graph.make_title_path(PORTFOLIO_PATH);

    let mut context = Portfolio {
        html_class: "portfolio",
        title: &section.title,
        title_path: &title_path,
        keywords: get_keywords(section),
        meta_description: &section.descr,
        flashed_messages: vec![],
        apache_docs: vec![],
        year: get_year(),
    };

    match read_apache_documents(&page_context.base_path) {
        Ok(docs) => {
            context.apache_docs = docs;
            context.into_response()
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            internal_server_error_page()
        }
    }
}

pub async fn serve_apache_document(
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let apache_documents = match read_apache_documents(&page_context.base_path) {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("{e:#?}");
            return internal_server_error_page();
        }
    };

    let map: HashMap<&str, &crate::domain::Apache> = apache_documents
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();

    let doc = path.trim_end_matches(".html");

    let Some(doc) = map.get(doc) else {
        return not_found_page();
    };

    let uri = format!("{PORTFOLIO_PATH}{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    let asset = ApacheTemplates::get(&path);
    if let Some(file) = asset {
        let content = String::from_utf8_lossy(&file.data);
        ApacheDocument {
            html_class: "",
            title: &doc.title,
            title_path: &title_path,
            keywords: &doc.keywords,
            meta_description: &doc.description,
            flashed_messages: vec![],
            content: &content,
            year: get_year(),
        }
        .into_response()
    } else {
        not_found_page()
    }
}

/// Gets downloadable files
#[utoipa::path(
    get,
    path = "/api/v2/portfolio/files/",
    params(),
    tag = "portfolio",
    responses(
        (status = 200, description = "Get files successfully", body = ApiResult<FilesContainer>),
    ),
)]
pub async fn serve_downloadable_files(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let downloads = read_downloads(page_context.clone()).await;
    if let Some(downloads) = downloads {
        let count = downloads.len() as i32;
        let result = ApiResult {
            result: downloads,
            pages: 1,
            page: 1,
            count,
            status: "success",
        };
        make_json_response(Ok(result)).into_response()
    } else {
        internal_server_error_page().into_response()
    }
}

pub fn read_apache_documents(base_path: &Path) -> Result<Vec<crate::domain::Apache>> {
    let config_path = base_path.join("apache/config.json");
    let file = File::open(config_path)
        .with_context(|| "Failed to open file that contain Apache documents configuration")?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)
        .with_context(|| "Failed to deserialize Apache documents configuration")?;
    Ok(result)
}

pub async fn redirect_to_real_document(
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let new_path = format!("/portfolio/{path}");
    Redirect::permanent(&new_path)
}

async fn read_downloads(page_context: Arc<PageContext<'_>>) -> Option<Vec<FilesContainer>> {
    let storage = page_context.storage.lock().await;

    let folders = storage.get_folders().ok()?;

    let mut result = vec![];
    for f in folders {
        let mut resource = Resource::new(&page_context.store_uri)?;
        let mut container = FilesContainer {
            title: f.title,
            ..Default::default()
        };
        resource.append_path("api").append_path(&f.bucket);
        let client = Client::builder()
            .timeout(Duration::from_secs(DOWNLOADS_WAIT_TIMEOUT_SECONDS))
            .build()
            .ok()?;

        match client.get(resource.to_string()).send().await {
            Ok(r) => {
                let files = r.json::<Vec<StoredFile>>().await;
                match files {
                    Ok(files) => {
                        for file in files {
                            match storage.get_download(file.id) {
                                Ok(meta_info) => {
                                    let downloadable = Downloadable {
                                        title: meta_info.title,
                                        path: format!("/storage/{}/{}", f.bucket, file.path),
                                        filename: file.path,
                                        size: file.size,
                                        blake3_hash: file.blake3_hash,
                                    };
                                    container.files.push(downloadable);
                                }
                                Err(e) => tracing::trace!("{e:#?}"),
                            }
                        }
                    }
                    Err(e) => tracing::error!("{e:#?}"),
                }
            }
            Err(e) => tracing::warn!("{e:#?}"),
        }
        result.push(container);
    }
    Some(result)
}

pub async fn serve_download_update(
    State(page_context): State<Arc<PageContext<'_>>>,
    Json(download): Json<Download>,
) -> impl IntoResponse {
    let mut storage = page_context.storage.lock().await;
    let result = storage.upsert_download(download);
    updated_response(result)
}

pub async fn serve_download_delete(
    extract::Path(id): extract::Path<i64>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> impl IntoResponse {
    let mut storage = page_context.storage.lock().await;
    let result = storage.delete_download(id);
    updated_response(result)
}

pub async fn serve_downloads_admin_api(
    State(page_context): State<Arc<PageContext<'_>>>,
    Query(request): Query<DownloadsRequest>,
) -> impl IntoResponse {
    let page_size = 10;
    let page = request.page.unwrap_or(1);
    let storage = page_context.storage.lock().await;

    let total_downloads_count = match storage.count_downloads() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("{e:#?}");
            return internal_server_error_page().into_response();
        }
    };

    let pages_count = count_pages(total_downloads_count, page_size);

    let downloads = match storage.get_downloads(page_size, page_size * (page - 1)) {
        Ok(downloads) => downloads,
        Err(e) => {
            tracing::error!("{e:#?}");
            return internal_server_error_page().into_response();
        }
    };

    let result = ApiResult {
        result: downloads,
        pages: pages_count,
        page,
        count: total_downloads_count,
        status: "success",
    };

    make_json_response(Ok(result)).into_response()
}

fn count_pages(count: i32, page_size: i32) -> i32 {
    count / page_size + i32::from(count % page_size > 0)
}
