use core::fmt::NumBuffer;

use crate::indie::ME;
use serde::Deserialize;
use serde_json::Value;
use url::Url;
use utoipa::IntoParams;

use super::{ApiError, Arc, Json, PageContext, Query, Result, State};

const GOOGLE_CUSTOM_SEARCH_URL: &str = "https://www.googleapis.com/customsearch/v1";

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchApiRequest {
    /// Search query string.
    pub q: Option<String>,
    /// 1-based index of the first result to return (Google CSE pagination).
    pub start: Option<u32>,
}

/// Proxies Google Custom Search so the API key stays server-side.
#[utoipa::path(
    get,
    path = "/api/v2/search/",
    params(SearchApiRequest),
    responses(
        (status = 200, description = "Search completed successfully"),
        (status = 400, description = "Missing or empty query"),
        (status = 500, description = "Search is not configured"),
        (status = 502, description = "Upstream search provider error"),
    ),
    tag = "search",
)]
pub async fn serve_search_api(
    State(page_context): State<Arc<PageContext<'_>>>,
    Query(request): Query<SearchApiRequest>,
) -> Result<Json<Value>, ApiError> {
    let q = request
        .q
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .ok_or_else(|| ApiError::bad_request("q is required"))?;

    let key = page_context.site_config.search_api_key.as_str();
    let cx = page_context.site_config.google_site_id.as_str();
    if key.is_empty() || cx.is_empty() {
        return Err(ApiError::internal(
            "search is not configured: missing API key or site id",
        ));
    }

    let start = request.start.unwrap_or(1).max(1);
    let url = google_search_url(key, cx, q, start)
        .ok_or_else(|| ApiError::internal("failed to build Google Custom Search URL"))?;

    // Keys restricted by HTTP referrer expect the site origin; browser used to send it.
    let response = page_context
        .http_client
        .get(url)
        .header("Referer", ME)
        .send()
        .await
        .map_err(|e| provider_error(anyhow::Error::from(e).context("request failed")))?;
    let status = response.status();
    let body = response
        .json::<Value>()
        .await
        .map_err(|e| provider_error(anyhow::Error::from(e).context("response cannot be parsed")))?;
    if !status.is_success() {
        let reason = google_error_summary(&body);
        return Err(provider_error(anyhow::anyhow!(
            "returned status {status}: {reason}"
        )));
    }
    Ok(Json(body))
}

fn provider_error(cause: anyhow::Error) -> ApiError {
    ApiError::bad_gateway("search provider error")
        .caused_by(cause.context("Google Custom Search failed"))
}

fn google_error_summary(body: &Value) -> String {
    body.get("error")
        .and_then(|error| {
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let reason = error
                .get("errors")
                .and_then(Value::as_array)
                .and_then(|errors| errors.first())
                .and_then(|entry| entry.get("reason"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            if message.is_empty() && reason.is_empty() {
                None
            } else if reason.is_empty() {
                Some(message.to_string())
            } else if message.is_empty() {
                Some(reason.to_string())
            } else {
                Some(format!("{reason}: {message}"))
            }
        })
        .unwrap_or_else(|| "no error details".to_string())
}

fn google_search_url(key: &str, cx: &str, q: &str, start: u32) -> Option<String> {
    let mut url = Url::parse(GOOGLE_CUSTOM_SEARCH_URL).ok()?;
    let mut buf = NumBuffer::new();
    url.query_pairs_mut()
        .append_pair("key", key)
        .append_pair("cx", cx)
        .append_pair("q", q)
        .append_pair("start", start.format_into(&mut buf));
    Some(url.to_string())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::google_search_url;
    use rstest::rstest;

    #[test]
    fn google_search_url_includes_encoded_query_params() {
        // Arrange
        let key = "secret-key";
        let cx = "site-cx";
        let q = "rust & axum";
        let start = 11;

        // Act
        let actual = google_search_url(key, cx, q, start).unwrap();

        // Assert
        assert!(actual.starts_with("https://www.googleapis.com/customsearch/v1?"));
        assert!(actual.contains("key=secret-key"));
        assert!(actual.contains("cx=site-cx"));
        assert!(actual.contains("q=rust+%26+axum") || actual.contains("q=rust%20%26%20axum"));
        assert!(actual.contains("start=11"));
    }

    #[rstest]
    #[case(1)]
    #[case(21)]
    fn google_search_url_preserves_start(#[case] start: u32) {
        // Arrange / Act
        let actual = google_search_url("k", "c", "q", start).unwrap();

        // Assert
        assert!(actual.contains(&format!("start={start}")));
    }

    #[test]
    fn google_error_summary_extracts_reason_and_message() {
        // Arrange
        let body = serde_json::json!({
            "error": {
                "code": 403,
                "message": "Requests from referer <empty> are blocked.",
                "errors": [{ "reason": "API_KEY_HTTP_REFERRER_BLOCKED" }]
            }
        });

        // Act
        let actual = super::google_error_summary(&body);

        // Assert
        assert_eq!(
            actual,
            "API_KEY_HTTP_REFERRER_BLOCKED: Requests from referer <empty> are blocked."
        );
    }
}
