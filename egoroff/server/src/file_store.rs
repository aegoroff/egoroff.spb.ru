use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use axum::body::Bytes;
use futures::{StreamExt, stream::BoxStream};
use kernel::resource::Resource;
use percent_encoding::percent_decode_str;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, de::DeserializeOwned};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Time for a whole metadata request (listing, last file).
/// Downloads and uploads are limited by the connect timeout only,
/// so large files are not cut off.
const METADATA_TIMEOUT: Duration = Duration::from_secs(5);

/// A file kept in the file store.
#[derive(Deserialize)]
pub struct StoredFile {
    pub id: i64,
    pub path: String,
    #[serde(default)]
    pub blake3_hash: String,
    pub size: u64,
}

/// Content of a file streamed from the file store.
pub struct FileBody {
    pub length: Option<u64>,
    pub stream: BoxStream<'static, reqwest::Result<Bytes>>,
}

/// Client of the external file store (`EGOROFF_STORE_URI`).
///
/// Knows the store's HTTP protocol, timeouts and response formats.
/// An unset or invalid store URI does not stop the server: every
/// operation then fails with an error.
#[derive(Clone)]
pub struct FileStore {
    base: Option<Resource>,
    client: Client,
}

impl FileStore {
    #[must_use]
    pub fn new(uri: &str) -> Self {
        let base = Resource::new(uri);
        if base.is_none() {
            tracing::warn!("Invalid file store uri '{uri}': file operations are disabled");
        }
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self { base, client }
    }

    pub async fn list(&self, bucket: &str) -> Result<Vec<StoredFile>> {
        self.get_json(&[bucket]).await
    }

    /// The most recently stored file of a bucket.
    pub async fn last(&self, bucket: &str) -> Result<StoredFile> {
        self.get_json(&[bucket, "last"]).await
    }

    /// Streams a file. Returns `None` when the store has no such file
    /// or a name is not a single safe path segment.
    pub async fn open(&self, bucket: &str, path: &str) -> Result<Option<FileBody>> {
        if !is_safe_path_segment(bucket) || !is_safe_path_segment(path) {
            return Ok(None);
        }
        let response = self.client.get(self.url(&[bucket, path])?).send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let response = response.error_for_status()?;
        Ok(Some(FileBody {
            length: response.content_length(),
            stream: response.bytes_stream().boxed(),
        }))
    }

    /// Uploads a file under `name` and returns the id the store gave it.
    pub async fn upload(&self, bucket: &str, name: &str, data: Vec<u8>) -> Result<i64> {
        let len = data.len() as u64;
        let part = reqwest::multipart::Part::stream_with_length(reqwest::Body::from(data), len)
            .file_name(name.to_owned());
        let form = reqwest::multipart::Form::new().part("file", part);

        let request = self.client.post(self.url(&[bucket, name])?).multipart(form);
        let ids: Vec<i64> = read_json(request).await?;
        ids.first()
            .copied()
            .context("file store returned no file id")
    }

    async fn get_json<T: DeserializeOwned>(&self, segments: &[&str]) -> Result<T> {
        let request = self
            .client
            .get(self.url(segments)?)
            .timeout(METADATA_TIMEOUT);
        read_json(request).await
    }

    fn url(&self, segments: &[&str]) -> Result<String> {
        let mut resource = self
            .base
            .clone()
            .ok_or_else(|| anyhow!("file store uri is not configured"))?;
        resource.append_path("api");
        for segment in segments {
            resource.append_path(segment);
        }
        Ok(resource.to_string())
    }
}

async fn read_json<T: DeserializeOwned>(request: RequestBuilder) -> Result<T> {
    Ok(request.send().await?.error_for_status()?.json().await?)
}

/// Guards against path traversal through the store proxy.
fn is_safe_path_segment(segment: &str) -> bool {
    let decoded = percent_decode_str(segment);
    let decoded = decoded.decode_utf8_lossy();
    if decoded.is_empty()
        || decoded.contains("..")
        || decoded.contains('/')
        || decoded.contains(':')
    {
        return false;
    }

    decoded
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use axum::{
        Json, Router,
        extract::{Multipart, Path},
        http::StatusCode as HttpStatus,
        response::IntoResponse,
        routing::get,
    };
    use rstest::rstest;
    use serde_json::json;

    /// Fake file store that speaks the store's HTTP protocol.
    ///
    /// Bucket `files` holds `a.txt`; bucket `broken` answers 500;
    /// bucket `empty` accepts uploads but returns no ids.
    async fn fake_store() -> FileStore {
        let app = Router::new()
            .route("/api/{bucket}", get(list))
            .route("/api/{bucket}/{name}", get(file).post(upload));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await });
        FileStore::new(&format!("http://{addr}/"))
    }

    async fn list(Path(bucket): Path<String>) -> impl IntoResponse {
        match bucket.as_str() {
            "files" => Json(json!([
                {"id": 1, "path": "a.txt", "blake3_hash": "abc", "size": 5},
                {"id": 2, "path": "b.txt", "blake3_hash": "def", "size": 7}
            ]))
            .into_response(),
            _ => HttpStatus::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    async fn file(Path((bucket, name)): Path<(String, String)>) -> impl IntoResponse {
        match (bucket.as_str(), name.as_str()) {
            ("files", "last") => Json(json!(
                {"id": 2, "path": "b.txt", "bucket": "files", "size": 7}
            ))
            .into_response(),
            ("files", "a.txt") => "hello".into_response(),
            ("broken", _) => HttpStatus::INTERNAL_SERVER_ERROR.into_response(),
            _ => HttpStatus::NOT_FOUND.into_response(),
        }
    }

    async fn upload(
        Path((bucket, _name)): Path<(String, String)>,
        mut multipart: Multipart,
    ) -> impl IntoResponse {
        let field = multipart.next_field().await.unwrap().unwrap();
        let data = field.bytes().await.unwrap();
        match bucket.as_str() {
            "files" => Json(json!([i64::try_from(data.len()).unwrap()])).into_response(),
            "empty" => Json(json!([])).into_response(),
            _ => HttpStatus::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    async fn read(body: FileBody) -> Vec<u8> {
        body.stream
            .map(|chunk| chunk.unwrap().to_vec())
            .concat()
            .await
    }

    #[tokio::test]
    async fn list_returns_files_of_bucket() {
        // arrange
        let store = fake_store().await;

        // act
        let files = store.list("files").await.unwrap();

        // assert
        let names: Vec<(i64, &str, &str, u64)> = files
            .iter()
            .map(|f| (f.id, f.path.as_str(), f.blake3_hash.as_str(), f.size))
            .collect();
        assert_eq!(vec![(1, "a.txt", "abc", 5), (2, "b.txt", "def", 7)], names);
    }

    #[tokio::test]
    async fn list_fails_when_store_fails() {
        // arrange
        let store = fake_store().await;

        // act
        let result = store.list("broken").await;

        // assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn last_returns_latest_file() {
        // arrange
        let store = fake_store().await;

        // act
        let file = store.last("files").await.unwrap();

        // assert
        assert_eq!((2, "b.txt"), (file.id, file.path.as_str()));
    }

    #[tokio::test]
    async fn open_streams_file_content() {
        // arrange
        let store = fake_store().await;

        // act
        let body = store.open("files", "a.txt").await.unwrap().unwrap();

        // assert
        assert_eq!(Some(5), body.length);
        assert_eq!(b"hello".to_vec(), read(body).await);
    }

    #[rstest]
    #[case("files", "missing.txt")]
    #[case("files", "..")]
    #[case("files", "a%2Fb")]
    #[case("..", "a.txt")]
    #[tokio::test]
    async fn open_misses_unknown_or_unsafe_files(#[case] bucket: &str, #[case] path: &str) {
        // arrange
        let store = fake_store().await;

        // act
        let body = store.open(bucket, path).await.unwrap();

        // assert
        assert!(body.is_none());
    }

    #[tokio::test]
    async fn open_fails_when_store_fails() {
        // arrange
        let store = fake_store().await;

        // act
        let result = store.open("broken", "a.txt").await;

        // assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn upload_returns_id_given_by_store() {
        // arrange
        let store = fake_store().await;

        // act
        let id = store
            .upload("files", "c.txt", b"123".to_vec())
            .await
            .unwrap();

        // assert
        assert_eq!(3, id);
    }

    #[rstest]
    #[case("empty")]
    #[case("broken")]
    #[tokio::test]
    async fn upload_fails_without_id(#[case] bucket: &str) {
        // arrange
        let store = fake_store().await;

        // act
        let result = store.upload(bucket, "c.txt", b"123".to_vec()).await;

        // assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn unconfigured_store_fails_every_operation() {
        // arrange
        let store = FileStore::new("");

        // act
        let list = store.list("files").await;
        let latest = store.last("files").await;
        let open = store.open("files", "a.txt").await;
        let upload = store.upload("files", "a.txt", vec![1]).await;

        // assert
        assert!(list.is_err());
        assert!(latest.is_err());
        assert!(open.is_err());
        assert!(upload.is_err());
    }

    #[rstest]
    #[case("123", true)]
    #[case("ab", true)]
    #[case("ab12", true)]
    #[case("ab-12", true)]
    #[case("ab_12", true)]
    #[case("ab_12.exe", true)]
    #[case("ab_1-2.exe", true)]
    #[case("ab12.", true)]
    #[case("ab12..", false)]
    #[case("ab..12", false)]
    #[case("ab/12", false)]
    #[case("", false)]
    #[case("qq:/", false)]
    #[case("qq:", false)]
    #[case("qq%3f", false)]
    #[trace]
    fn is_safe_path_segment_tests(#[case] test_data: &str, #[case] expected: bool) {
        // arrange
        // act
        let actual = is_safe_path_segment(test_data);

        // assert
        assert_eq!(expected, actual);
    }
}
