use std::time::Duration;

use anyhow::Context;
use kernel::{
    domain::{ApiResult, Download, DownloadsRequest, Folder},
    paging,
};
use serde::Deserialize;

use crate::domain::{Downloadable, FilesContainer};
use axum::response::Redirect;

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
        let count = i32::try_from(downloads.len()).unwrap_or(i32::MAX);
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

/// Reads downloadable files of all folders.
///
/// The storage lock is not held while the file store is queried over HTTP,
/// so a slow file store cannot stall other requests that need the database.
async fn read_downloads(page_context: Arc<PageContext<'_>>) -> Option<Vec<FilesContainer>> {
    let folders = page_context.storage.lock().await.get_folders().ok()?;

    let base = Resource::new(&page_context.store_uri)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(DOWNLOADS_WAIT_TIMEOUT_SECONDS))
        .build()
        .ok()?;

    let mut listings = Vec::with_capacity(folders.len());
    for folder in folders {
        let files = fetch_stored_files(&client, base.clone(), &folder.bucket).await;
        listings.push((folder, files));
    }

    let storage = page_context.storage.lock().await;
    Some(make_files_containers(&*storage, listings))
}

async fn fetch_stored_files(
    client: &Client,
    mut resource: Resource,
    bucket: &str,
) -> Vec<StoredFile> {
    resource.append_path("api").append_path(bucket);
    match client.get(resource.to_string()).send().await {
        Ok(r) => r.json::<Vec<StoredFile>>().await.unwrap_or_else(|e| {
            tracing::error!("{e:#?}");
            vec![]
        }),
        Err(e) => {
            tracing::warn!("{e:#?}");
            vec![]
        }
    }
}

/// Combines file store listings with download titles from the database.
/// Files without a database record are skipped.
fn make_files_containers(
    storage: &impl Storage,
    listings: Vec<(Folder, Vec<StoredFile>)>,
) -> Vec<FilesContainer> {
    listings
        .into_iter()
        .map(|(folder, files)| {
            let files = files
                .into_iter()
                .filter_map(|file| match storage.get_download(file.id) {
                    Ok(meta_info) => Some(Downloadable {
                        title: meta_info.title,
                        path: format!("/storage/{}/{}", folder.bucket, file.path),
                        filename: file.path,
                        size: file.size,
                        blake3_hash: file.blake3_hash,
                    }),
                    Err(e) => {
                        tracing::trace!("{e:#?}");
                        None
                    }
                })
                .collect();
            FilesContainer {
                title: folder.title,
                files,
            }
        })
        .collect()
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
    let (page, offset) = paging::page_offset(request.page, page_size);
    let storage = page_context.storage.lock().await;

    let total_downloads_count = match storage.count_downloads() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("{e:#?}");
            return internal_server_error_page().into_response();
        }
    };

    let pages_count = paging::pages_count(total_downloads_count, page_size);

    let downloads = match storage.get_downloads(page_size, offset) {
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use kernel::sqlite::{Mode, Sqlite};

    fn stored_file(id: i64, path: &str) -> StoredFile {
        StoredFile {
            id,
            path: path.to_string(),
            blake3_hash: format!("hash{id}"),
            size: 10,
        }
    }

    #[test]
    fn make_files_containers_joins_titles_and_skips_unknown_files() {
        // arrange
        let mut storage = Sqlite::open(":memory:", Mode::ReadWrite).unwrap();
        storage.new_database().unwrap();
        storage
            .upsert_download(Download {
                id: 1,
                title: String::from("Known file"),
            })
            .unwrap();
        let folder = Folder {
            bucket: String::from("apps"),
            title: String::from("Applications"),
        };
        let files = vec![stored_file(1, "known.zip"), stored_file(2, "unknown.zip")];

        // act
        let actual = make_files_containers(&storage, vec![(folder, files)]);

        // assert
        assert_eq!(1, actual.len());
        assert_eq!("Applications", actual[0].title);
        assert_eq!(1, actual[0].files.len());
        let file = &actual[0].files[0];
        assert_eq!("Known file", file.title);
        assert_eq!("/storage/apps/known.zip", file.path);
        assert_eq!("known.zip", file.filename);
        assert_eq!("hash1", file.blake3_hash);
    }

    #[test]
    fn make_files_containers_keeps_folder_without_files() {
        // arrange
        let storage = Sqlite::open(":memory:", Mode::ReadWrite).unwrap();
        storage.new_database().unwrap();
        let folder = Folder {
            bucket: String::from("empty"),
            title: String::from("Empty"),
        };

        // act
        let actual = make_files_containers(&storage, vec![(folder, vec![])]);

        // assert
        assert_eq!(1, actual.len());
        assert!(actual[0].files.is_empty());
    }
}
