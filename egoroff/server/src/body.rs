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
