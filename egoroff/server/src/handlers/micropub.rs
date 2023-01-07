use serde::Deserialize;

use crate::indie::ME;

use super::*;

#[derive(Deserialize, Serialize)]
pub struct MicropubRequest {
    pub q: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize)]
pub struct MicropubConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<Vec<String>>,
    #[serde(
        rename(serialize = "media-endpoint"),
        skip_serializing_if = "Option::is_none"
    )]
    pub media_endpoint: Option<String>,
    #[serde(
        rename(serialize = "syndicate-to"),
        skip_serializing_if = "Option::is_none"
    )]
    pub syndicate_to: Option<Vec<SyndicateTo>>,
}

#[derive(Serialize)]
pub struct SyndicateTo {
    pub uid: String,
    pub name: String,
}

pub async fn serve_index(Query(query): Query<MicropubRequest>) -> impl IntoResponse {
    if let Some(q) = query.q {
        match q.as_str() {
            "config" => {
                let config = MicropubConfig {
                    q: Some(vec![
                        "config".to_string(),
                        "media-endpoint".to_string(),
                        "source".to_string(),
                        "syndicate-to".to_string(),
                    ]),
                    media_endpoint: Some(ME.to_string() + "micropub/media"),
                    syndicate_to: Some(vec![]),
                };
                (StatusCode::OK, Json(config).into_response())
            }
            "media-endpoint" => {
                let config = MicropubConfig {
                    q: None,
                    media_endpoint: Some(ME.to_string() + "micropub/media"),
                    syndicate_to: None,
                };
                (StatusCode::OK, Json(config).into_response())
            }
            "syndicate-to" => {
                let config = MicropubConfig {
                    q: None,
                    media_endpoint: None,
                    syndicate_to: Some(vec![]),
                };
                (StatusCode::OK, Json(config).into_response())
            }
            _ => (StatusCode::OK, Empty::new().into_response()),
        }
    } else {
        (StatusCode::OK, Empty::new().into_response())
    }
}
