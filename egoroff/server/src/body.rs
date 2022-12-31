use std::path::Path;

use axum::{
    body::{Bytes, Full, StreamBody},
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
    BoxError,
};
use futures::TryStream;

/// Custom response with content type specified.
#[derive(Clone, Copy, Debug)]
pub struct Content<T>(pub T, pub &'static str);

impl<T> IntoResponse for Content<T>
where
    T: Into<Full<Bytes>>,
{
    fn into_response(self) -> Response {
        (
            [(header::CONTENT_TYPE, HeaderValue::from_static(self.1))],
            self.0.into(),
        )
            .into_response()
    }
}

/// An XML response.
///
/// Will automatically get `Content-Type: text/xml`.
#[derive(Clone, Copy, Debug)]
pub struct Xml<T>(pub T);

impl<T> IntoResponse for Xml<T>
where
    T: Into<Full<Bytes>>,
{
    fn into_response(self) -> Response {
        Content(self.0, "text/xml; charset=utf-8").into_response()
    }
}

impl<T> From<T> for Xml<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

/// An HTML response.
///
/// Will automatically get `Content-Type: text/html`.
#[derive(Clone, Copy, Debug)]
pub struct Html<T>(pub T);

impl<T> IntoResponse for Html<T>
where
    T: Into<Full<Bytes>>,
{
    fn into_response(self) -> Response {
        (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                ),
                (
                    header::X_XSS_PROTECTION,
                    HeaderValue::from_static("1; mode=block"),
                ),
            ],
            self.0.into(),
        )
            .into_response()
    }
}

impl<T> From<T> for Html<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

/// An Binary response.
///
/// Will automatically get content type using file path.
#[derive(Clone, Copy, Debug)]
pub struct Binary<T, P> {
    data: T,
    path: P,
}

impl<T, P> Binary<T, P>
where
    T: Into<Full<Bytes>>,
    P: AsRef<Path>,
{
    pub fn new(data: T, path: P) -> Self {
        Self { data, path }
    }
}

impl<T, P> IntoResponse for Binary<T, P>
where
    T: Into<Full<Bytes>>,
    P: AsRef<Path>,
{
    fn into_response(self) -> Response {
        let mime = mime_guess::from_path(self.path).first_or_octet_stream();
        (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime.as_ref()).unwrap(),
                ),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=315360000"),
                ),
                (
                    header::EXPIRES,
                    HeaderValue::from_static("Thu, 31 Dec 2037 23:55:55 GMT"),
                ),
            ],
            self.data.into(),
        )
            .into_response()
    }
}

pub struct FileReply<S> {
    data: S,
    path: String,
    length: Option<i64>,
}

impl<S> FileReply<S>
where
    S: TryStream + Send + 'static,
    S::Ok: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    pub fn new(data: S, path: String, length: Option<i64>) -> Self {
        Self { data, path, length }
    }

    fn name_from_path(&self) -> &str {
        let path = &self.path;
        if let Some(ix) = path.rfind('\\') {
            &path[ix + 1..]
        } else if let Some(ix) = path.rfind('/') {
            &path[ix + 1..]
        } else {
            path
        }
    }
}

impl<S> IntoResponse for FileReply<S>
where
    S: TryStream + Send + 'static,
    S::Ok: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    fn into_response(self) -> Response {
        let file_name = self.name_from_path().to_owned();
        let attachment = format!(r#"attachment; filename="{file_name}""#);
        let body = StreamBody::new(self.data);
        let content_type = (
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );

        let content_disposition = (
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(attachment.as_str()).unwrap(),
        );
        if let Some(len) = self.length {
            let content_length = (
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&len.to_string()).unwrap(),
            );
            ([content_type, content_disposition, content_length], body).into_response()
        } else {
            ([content_type, content_disposition], body).into_response()
        }
        .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("", "")]
    #[case("file.ext", "file.ext")]
    #[case("dir/file.ext", "file.ext")]
    #[case("dir\\file.ext", "file.ext")]
    #[case("dir1\\dir2\\file.ext", "file.ext")]
    #[case("dir1/dir2/file.ext", "file.ext")]
    #[trace]
    fn name_from_path(#[case] path: &str, #[case] expected: &str) {
        // Arrange
        let data = b"hello, world!";
        let stream = tokio_util::io::ReaderStream::new(&data[..]);
        let reply = FileReply::new(stream, path.to_string(), None);

        // Act
        let name = reply.name_from_path();

        // Assert
        assert_eq!(name, expected);
    }
}
