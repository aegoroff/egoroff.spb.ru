use std::marker::PhantomData;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::{get_year, template::ErrorPage};
use crate::domain::{Error, OperationResult};

const INTERNAL_ERROR_TEXT: &str = "internal server error";

/// Failure of a handler that renders an HTML page.
/// Responds with the site's error page; the cause, if any, is logged and never shown.
#[derive(Debug)]
pub struct PageError {
    status: StatusCode,
    cause: Option<anyhow::Error>,
}

/// Failure of a handler that serves an API, a protocol endpoint or a file.
/// Responds with plain text; the cause, if any, is logged. Internal failures
/// get a generic text so their details stay in the log.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    text: String,
    cause: Option<anyhow::Error>,
}

/// Failure of a JSON endpoint that returns `T`.
/// Responds with 500 and an empty `T`, so clients always get the JSON shape they expect.
#[derive(Debug)]
pub struct JsonError<T>(anyhow::Error, PhantomData<T>);

/// Failure of an admin create, update or delete.
/// Responds with 500 and `{"result": "<error text>"}`.
#[derive(Debug)]
pub enum OperationError {
    Unauthorized,
    Failed(anyhow::Error),
}

pub type JsonResult<T> = Result<Json<T>, JsonError<T>>;
pub type OperationResponse = Result<Json<OperationResult<'static>>, OperationError>;

impl PageError {
    pub fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            cause: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::failed(failure(message))
    }

    /// Attaches the underlying error; it is logged when the response is built.
    #[must_use]
    pub fn caused_by(mut self, cause: impl Into<anyhow::Error>) -> Self {
        self.cause = Some(cause.into());
        self
    }

    fn failed(cause: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            cause: Some(cause),
        }
    }
}

impl ApiError {
    pub fn bad_request(text: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, text)
    }

    pub fn unauthorized(text: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, text)
    }

    pub fn not_found(text: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, text)
    }

    pub fn bad_gateway(text: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_GATEWAY, text)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::failed(failure(message))
    }

    /// Attaches the underlying error; it is logged when the response is built.
    #[must_use]
    pub fn caused_by(mut self, cause: impl Into<anyhow::Error>) -> Self {
        self.cause = Some(cause.into());
        self
    }

    /// Logs the response text itself when there is no underlying error.
    #[must_use]
    pub fn logged(self) -> Self {
        let cause = failure(self.text.clone());
        self.caused_by(cause)
    }

    fn new(status: StatusCode, text: impl Into<String>) -> Self {
        Self {
            status,
            text: text.into(),
            cause: None,
        }
    }

    fn failed(cause: anyhow::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, INTERNAL_ERROR_TEXT).caused_by(cause)
    }
}

impl<T> JsonError<T> {
    fn new(cause: anyhow::Error) -> Self {
        Self(cause, PhantomData)
    }
}

fn failure(message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::msg(message.into())
}

/// Lets handlers use `?` on any error that converts into `anyhow::Error`.
macro_rules! from_any_error {
    ($target:ty, $wrap:path $(, $generic:ident)?) => {
        impl<$($generic,)? E: Into<anyhow::Error>> From<E> for $target {
            fn from(e: E) -> Self {
                $wrap(e.into())
            }
        }
    };
}

from_any_error!(PageError, PageError::failed);
from_any_error!(ApiError, ApiError::failed);
from_any_error!(JsonError<T>, JsonError::new, T);
from_any_error!(OperationError, OperationError::Failed);

fn log_cause(cause: Option<anyhow::Error>) {
    if let Some(cause) = cause {
        tracing::error!("{cause:#?}");
    }
}

impl IntoResponse for PageError {
    fn into_response(self) -> Response {
        log_cause(self.cause);
        let code = self.status.as_u16().to_string();
        let page = ErrorPage {
            html_class: "",
            title: &code,
            title_path: "",
            keywords: "",
            meta_description: "",
            error: Error {
                code: code.clone(),
                ..Default::default()
            },
            year: get_year(),
        };
        (self.status, page).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        log_cause(self.cause);
        (self.status, self.text).into_response()
    }
}

impl<T: Default + Serialize> IntoResponse for JsonError<T> {
    fn into_response(self) -> Response {
        log_cause(Some(self.0));
        (StatusCode::INTERNAL_SERVER_ERROR, Json(T::default())).into_response()
    }
}

impl IntoResponse for OperationError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Self::Failed(e) => {
                let result = e.to_string();
                log_cause(Some(e));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(OperationResult { result: &result }),
                )
                    .into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use axum::body::to_bytes;
    use rstest::rstest;

    async fn body_text(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[rstest]
    #[case(
        ApiError::bad_request("bad input"),
        StatusCode::BAD_REQUEST,
        "bad input"
    )]
    #[case(
        ApiError::unauthorized("no token"),
        StatusCode::UNAUTHORIZED,
        "no token"
    )]
    #[case(
        ApiError::not_found("a/b not found"),
        StatusCode::NOT_FOUND,
        "a/b not found"
    )]
    #[case(ApiError::bad_gateway("upstream"), StatusCode::BAD_GATEWAY, "upstream")]
    #[case(
        ApiError::bad_request("bad input").caused_by(anyhow::anyhow!("parser detail")),
        StatusCode::BAD_REQUEST,
        "bad input"
    )]
    #[case(ApiError::bad_request("bad input").logged(), StatusCode::BAD_REQUEST, "bad input")]
    #[tokio::test]
    async fn api_error_responds_with_status_and_text(
        #[case] error: ApiError,
        #[case] status: StatusCode,
        #[case] text: &str,
    ) {
        // arrange

        // act
        let response = error.into_response();

        // assert
        assert_eq!(status, response.status());
        assert_eq!(text, body_text(response).await);
    }

    #[rstest]
    #[case(ApiError::from(anyhow::anyhow!("secret sql details")))]
    #[case(ApiError::internal("secret config details"))]
    #[tokio::test]
    async fn api_internal_error_hides_details(#[case] error: ApiError) {
        // arrange

        // act
        let response = error.into_response();

        // assert
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
        assert_eq!(INTERNAL_ERROR_TEXT, body_text(response).await);
    }

    #[rstest]
    #[case(PageError::not_found(), StatusCode::NOT_FOUND)]
    #[case(
        PageError::not_found().caused_by(anyhow::anyhow!("broken")),
        StatusCode::NOT_FOUND
    )]
    #[case(PageError::internal("broken"), StatusCode::INTERNAL_SERVER_ERROR)]
    #[case(
        PageError::from(anyhow::anyhow!("broken")),
        StatusCode::INTERNAL_SERVER_ERROR
    )]
    #[tokio::test]
    async fn page_error_renders_error_page(#[case] error: PageError, #[case] status: StatusCode) {
        // arrange

        // act
        let response = error.into_response();

        // assert
        assert_eq!(status, response.status());
        let html = body_text(response).await;
        assert!(html.contains(status.as_str()));
        assert!(!html.contains("broken"));
    }

    #[derive(Serialize, Default)]
    struct Listing {
        items: Vec<i32>,
        total: i32,
    }

    #[tokio::test]
    async fn json_error_responds_with_empty_json() {
        // arrange
        let error = JsonError::<Listing>::from(anyhow::anyhow!("db is down"));

        // act
        let response = error.into_response();

        // assert
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
        assert_eq!(r#"{"items":[],"total":0}"#, body_text(response).await);
    }

    #[tokio::test]
    async fn operation_error_reports_error_text_as_result() {
        // arrange
        let error = OperationError::from(anyhow::anyhow!("UNIQUE constraint failed"));

        // act
        let response = error.into_response();

        // assert
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
        assert_eq!(
            r#"{"result":"UNIQUE constraint failed"}"#,
            body_text(response).await
        );
    }

    #[tokio::test]
    async fn operation_unauthorized_has_empty_body() {
        // arrange
        let error = OperationError::Unauthorized;

        // act
        let response = error.into_response();

        // assert
        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
        assert!(body_text(response).await.is_empty());
    }
}
