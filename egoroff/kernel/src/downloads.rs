use std::collections::HashMap;

use anyhow::Result;

use crate::{
    database::{Database, is_not_found},
    domain::{ApiResult, Download, Folder, Storage},
    paging::{page_offset, pages_count},
};

/// Downloads per page in the admin listing.
pub const ADMIN_PAGE_SIZE: i32 = 10;

/// Download folders and the titles of the files kept in the file store.
#[derive(Clone)]
pub struct Downloads {
    db: Database,
}

impl Downloads {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn folders(&self) -> Result<Vec<Folder>> {
        self.db.call(|storage| storage.get_folders()).await
    }

    /// Titles of the downloads with the given file ids; ids without a
    /// download record are left out.
    pub async fn titles(&self, ids: Vec<i64>) -> Result<HashMap<i64, String>> {
        self.db
            .call(move |storage| {
                let mut titles = HashMap::with_capacity(ids.len());
                for id in ids {
                    match storage.get_download(id) {
                        Ok(download) => {
                            titles.insert(id, download.title);
                        }
                        Err(e) if is_not_found(&e) => {}
                        Err(e) => return Err(e),
                    }
                }
                Ok(titles)
            })
            .await
    }

    /// One page of downloads, newest first.
    pub async fn admin_page(&self, page: Option<i32>) -> Result<ApiResult<Download>> {
        let (page, offset) = page_offset(page, ADMIN_PAGE_SIZE);
        let (count, downloads) = self
            .db
            .call(move |storage| {
                Ok((
                    storage.count_downloads()?,
                    storage.get_downloads(ADMIN_PAGE_SIZE, offset)?,
                ))
            })
            .await?;

        Ok(ApiResult {
            result: downloads,
            pages: pages_count(count, ADMIN_PAGE_SIZE),
            page,
            count,
            status: "success",
        })
    }

    pub async fn update(&self, download: Download) -> Result<()> {
        self.db
            .call(|storage| storage.upsert_download(download))
            .await
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        self.db
            .call(move |storage| storage.delete_download(id))
            .await?;
        Ok(())
    }

    pub async fn count(&self) -> Result<i32> {
        self.db.call(|storage| storage.count_downloads()).await
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::sqlite::{Mode, Sqlite};
    use rstest::{fixture, rstest};

    #[fixture]
    fn downloads() -> Downloads {
        let storage = Sqlite::open(":memory:", Mode::ReadWrite).unwrap();
        Downloads::new(Database::new(storage))
    }

    fn download(id: i64) -> Download {
        Download {
            id,
            title: format!("File {id}"),
        }
    }

    async fn store(downloads: &Downloads, ids: impl IntoIterator<Item = i64>) {
        for id in ids {
            downloads.update(download(id)).await.unwrap();
        }
    }

    #[rstest]
    #[tokio::test]
    async fn titles_leave_out_files_without_download(downloads: Downloads) {
        // arrange
        store(&downloads, [1, 3]).await;

        // act
        let titles = downloads.titles(vec![1, 2, 3]).await.unwrap();

        // assert
        let expected: HashMap<i64, String> = [(1, "File 1".into()), (3, "File 3".into())].into();
        assert_eq!(expected, titles);
    }

    #[rstest]
    #[case(None, 1, vec![12, 11, 10, 9, 8, 7, 6, 5, 4, 3])]
    #[case(Some(2), 2, vec![2, 1])]
    #[tokio::test]
    async fn admin_page_lists_newest_first_by_page(
        downloads: Downloads,
        #[case] requested: Option<i32>,
        #[case] expected_page: i32,
        #[case] expected_ids: Vec<i64>,
    ) {
        // arrange
        store(&downloads, 1..=12).await;

        // act
        let page = downloads.admin_page(requested).await.unwrap();

        // assert
        let ids: Vec<i64> = page.result.iter().map(|d| d.id).collect();
        assert_eq!(expected_ids, ids);
        assert_eq!(expected_page, page.page);
        assert_eq!(2, page.pages);
        assert_eq!(12, page.count);
    }

    #[rstest]
    #[tokio::test]
    async fn update_replaces_title(downloads: Downloads) {
        // arrange
        store(&downloads, [1]).await;

        // act
        downloads
            .update(Download {
                id: 1,
                title: String::from("Renamed"),
            })
            .await
            .unwrap();

        // assert
        let titles = downloads.titles(vec![1]).await.unwrap();
        assert_eq!(Some(&String::from("Renamed")), titles.get(&1));
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_download(downloads: Downloads) {
        // arrange
        store(&downloads, [1, 2]).await;

        // act
        downloads.delete(1).await.unwrap();

        // assert
        assert_eq!(1, downloads.count().await.unwrap());
        assert!(downloads.titles(vec![1]).await.unwrap().is_empty());
    }
}
