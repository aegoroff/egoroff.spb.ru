use std::path::PathBuf;

use kernel::{
    domain::{ApiResult, SmallPost},
    graph::{SiteGraph, SiteSection},
};
use oauth2::CsrfToken;
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

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub scope: Option<String>,
    pub state: CsrfToken,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AuthorizedUser {
    pub login_or_name: String,
    pub authenticated: bool,
    pub admin: bool,
    pub provider: String,
}

impl Poster {
    pub fn new(api: ApiResult, page: i32) -> Self {
        let pages_count = api.pages;
        let prev_page = if page == 1 { 1 } else { page - 1 };
        let next_page = if page == pages_count {
            pages_count
        } else {
            page + 1
        };
        let pages: Vec<i32> = (1..=pages_count).collect();
        Self {
            small_posts: api.result,
            pages,
            has_pages: pages_count > 1,
            has_prev: page > 1,
            has_next: page < pages_count,
            page,
            prev_page,
            next_page,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poster_new_with_pages_first_page() {
        // arrange
        let api_result = ApiResult {
            result: vec![],
            pages: 2,
            page: 1,
            count: 40,
            status: String::new(),
        };
        let page = 1;

        // act
        let poster = Poster::new(api_result, page);

        // assert
        assert!(poster.has_pages);
        assert!(poster.has_next);
        assert!(!poster.has_prev);
        assert_eq!(poster.pages, vec![1, 2]);
        assert_eq!(poster.prev_page, 1);
        assert_eq!(poster.next_page, 2);
    }

    #[test]
    fn poster_new_with_pages_last_page() {
        // arrange
        let api_result = ApiResult {
            result: vec![],
            pages: 2,
            page: 1,
            count: 40,
            status: String::new(),
        };
        let page = 2;

        // act
        let poster = Poster::new(api_result, page);

        // assert
        assert!(poster.has_pages);
        assert!(!poster.has_next);
        assert!(poster.has_prev);
        assert_eq!(poster.pages, vec![1, 2]);
        assert_eq!(poster.prev_page, 1);
        assert_eq!(poster.next_page, 2);
    }

    #[test]
    fn poster_new_with_pages_middle_page() {
        // arrange
        let api_result = ApiResult {
            result: vec![],
            pages: 3,
            page: 1,
            count: 60,
            status: String::new(),
        };
        let page = 2;

        // act
        let poster = Poster::new(api_result, page);

        // assert
        assert!(poster.has_pages);
        assert!(poster.has_next);
        assert!(poster.has_prev);
        assert_eq!(poster.pages, vec![1, 2, 3]);
        assert_eq!(poster.prev_page, 1);
        assert_eq!(poster.next_page, 3);
    }

    #[test]
    fn poster_new_without_pages() {
        // arrange
        let api_result = ApiResult {
            result: vec![],
            pages: 1,
            page: 1,
            count: 20,
            status: String::new(),
        };
        let page = 1;

        // act
        let poster = Poster::new(api_result, page);

        // assert
        assert!(!poster.has_pages);
        assert!(!poster.has_next);
        assert!(!poster.has_prev);
        assert_eq!(poster.pages, vec![1]);
        assert_eq!(poster.prev_page, 1);
        assert_eq!(poster.next_page, 1);
    }
}
