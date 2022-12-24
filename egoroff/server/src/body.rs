use std::path::Path;

use axum::{
    body::{Bytes, Full},
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
};

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
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/xml; charset=utf-8"),
            )],
            self.0.into(),
        )
            .into_response()
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
