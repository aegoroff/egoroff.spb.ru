use std::path::PathBuf;

use kernel::{
    domain::SmallPost,
    graph::{SiteGraph, SiteSection},
};
use serde::{Deserialize, Serialize};
use tera::Tera;

#[derive(Deserialize)]
pub struct Uri {
    pub uri: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct BlogRequest {
    pub tag: Option<String>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Navigation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<SiteSection>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub breadcrumbs: Option<Vec<SiteSection>>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub search_api_key: String,
    pub google_site_id: String,
    pub analytics_id: String,
}

#[derive(Clone)]
pub struct PageContext {
    pub base_path: PathBuf,
    pub storage_path: PathBuf,
    pub tera: Tera,
    pub site_graph: SiteGraph,
    pub site_config: Config,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Apache {
    pub id: String,
    pub stylesheet: String,
    pub title: String,
    pub description: String,
    pub keywords: String,
}

#[derive(Serialize, Default)]
pub struct Poster {
    pub small_posts: Vec<SmallPost>,
    pub pages: Vec<i32>,
    pub has_pages: bool,
    pub has_prev: bool,
    pub has_next: bool,
    pub page: i32,
    pub prev_page: i32,
    pub next_page: i32,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Error {
    pub code: String,
    pub name: String,
}
