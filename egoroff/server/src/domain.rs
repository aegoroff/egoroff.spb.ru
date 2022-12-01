use kernel::{graph::SiteSection, domain::SmallPost};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Uri {
    pub uri: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Navigation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<SiteSection>>,
}

#[derive(Serialize, Default)]
pub struct Config {
    pub search_api_key: String,
    pub google_site_id: String,
}

#[derive(Serialize, Default)]
pub struct Poster {
    pub small_posts: Vec<SmallPost>,
}