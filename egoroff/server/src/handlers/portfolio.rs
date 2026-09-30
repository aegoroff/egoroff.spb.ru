use anyhow::Context;
use kernel::{
    domain::{ApiResult, Download, DownloadsRequest, Folder},
    paging,
};

use crate::domain::{Downloadable, FilesContainer};
use crate::file_store::StoredFile;
use axum::response::Redirect;

use super::{
    Arc, BufReader, File, IntoResponse, Json, JsonResult, OperationResponse, PageContext,
    PageError, Path, Query, Response, Result, RustEmbed, State, Storage, extract, find_section,
    get_keywords, get_year,
    template::{ApacheDocument, Portfolio},
    updated,
};

#[derive(RustEmbed)]
#[folder = "../../templates/apache"]
struct ApacheTemplates;

const PORTFOLIO_PATH: &str = "/portfolio/";

pub async fn serve_index(
    State(page_context): State<Arc<PageContext<'_>>>,
) -> Result<Response, PageError> {
    let section = find_section(&page_context, "portfolio")?;
    let title_path = page_context.site_graph.make_title_path(PORTFOLIO_PATH);

    Ok(Portfolio {
        html_class: "portfolio",
        title: &section.title,
        title_path: &title_path,
        keywords: get_keywords(section),
        meta_description: &section.descr,
        apache_docs: &page_context.apache_docs,
        year: get_year(),
    }
    .into_response())
}

pub async fn serve_apache_document(
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(path): extract::Path<String>,
) -> Result<Response, PageError> {
    let id = path.trim_end_matches(".html");
    let doc = page_context
        .apache_docs
        .iter()
        .find(|item| item.id == id)
        .ok_or_else(PageError::not_found)?;
    let file = ApacheTemplates::get(&path).ok_or_else(PageError::not_found)?;

    let uri = format!("{PORTFOLIO_PATH}{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);
    let content = String::from_utf8_lossy(&file.data);
    Ok(ApacheDocument {
        html_class: "",
        title: &doc.title,
        title_path: &title_path,
        keywords: &doc.keywords,
        meta_description: &doc.description,
        content: &content,
        year: get_year(),
    }
    .into_response())
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
) -> JsonResult<ApiResult<FilesContainer>> {
    let downloads = read_downloads(page_context.clone()).await?;
    let count = i32::try_from(downloads.len()).unwrap_or(i32::MAX);
    Ok(Json(ApiResult {
        result: downloads,
        pages: 1,
        page: 1,
        count,
        status: "success",
    }))
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
async fn read_downloads(page_context: Arc<PageContext<'_>>) -> Result<Vec<FilesContainer>> {
    let folders = page_context.storage.lock().await.get_folders()?;

    let mut listings = Vec::with_capacity(folders.len());
    for folder in folders {
        let files = page_context
            .file_store
            .list(&folder.bucket)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("{e:#?}");
                vec![]
            });
        listings.push((folder, files));
    }

    let storage = page_context.storage.lock().await;
    Ok(make_files_containers(&*storage, listings))
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
) -> OperationResponse {
    page_context
        .storage
        .lock()
        .await
        .upsert_download(download)?;
    Ok(updated())
}

pub async fn serve_download_delete(
    extract::Path(id): extract::Path<i64>,
    State(page_context): State<Arc<PageContext<'_>>>,
) -> OperationResponse {
    page_context.storage.lock().await.delete_download(id)?;
    Ok(updated())
}

pub async fn serve_downloads_admin_api(
    State(page_context): State<Arc<PageContext<'_>>>,
    Query(request): Query<DownloadsRequest>,
) -> JsonResult<ApiResult<Download>> {
    let page_size = 10;
    let (page, offset) = paging::page_offset(request.page, page_size);
    let storage = page_context.storage.lock().await;
    let count = storage.count_downloads()?;
    let downloads = storage.get_downloads(page_size, offset)?;

    Ok(Json(ApiResult {
        result: downloads,
        pages: paging::pages_count(count, page_size),
        page,
        count,
        status: "success",
    }))
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
