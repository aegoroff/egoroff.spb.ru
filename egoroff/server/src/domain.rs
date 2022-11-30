use kernel::graph::SiteSection;
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