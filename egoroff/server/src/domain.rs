use std::{collections::HashSet, path::PathBuf, sync::Arc};

use futures::lock::Mutex;
use kernel::{
    domain::ApiResult,
    graph::{SiteGraph, SiteSection},
    sqlite::Sqlite,
};
use oauth2::CsrfToken;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub type Database = Arc<Mutex<Sqlite>>;
pub type Cache = Arc<Mutex<HashSet<String>>>;

/// Represents a URI, which is a string representing a Uniform Resource Identifier.
#[derive(Deserialize)]
pub struct Uri {
   /// The actual URI value.
   pub uri: String,
}

/// Represents the result of an operation. It holds a reference to a string result.
#[derive(Deserialize, Serialize, Default)]
pub struct OperationResult<'a> {
   /// The result of the operation as a string reference.
   pub result: &'a str,
}

/// Represents a request for a blog.
#[derive(Deserialize, Serialize, Default)]
pub struct BlogRequest {
   /// The tag associated with the blog request (optional).
   pub tag: Option<String>,
}

/// Represents navigation data in the application.
#[derive(Deserialize, Serialize, Default)]
pub struct Navigation {
   /// A list of site sections (optional).
   #[serde(skip_serializing_if = "Option::is_none")]
   pub sections: Option<Vec<SiteSection>>,

   /// A list of breadcrumbs (optional).
   #[serde(skip_serializing_if = "Option::is_none")]
   pub breadcrumbs: Option<Vec<SiteSection>>,
}

/// Represents the application's configuration data.
#[derive(Serialize, Deserialize, Default)]
pub struct Config {
   /// The search API key.
   pub search_api_key: String,
   /// The Google Site ID.
   pub google_site_id: String,
   /// The analytics ID.
   pub analytics_id: String,
}

/// Represents the context of a page in the application.
pub struct PageContext<'a> {
   /// The base path of the page.
   pub base_path: PathBuf,
   /// The database storage instance.
   pub storage: Database,
   /// The site graph instance.
   pub site_graph: Arc<SiteGraph<'a>>,
   /// The site configuration data.
   pub site_config: Config,
   /// The store URI.
   pub store_uri: String,
   /// The certificates path.
   pub certs_path: String,
   /// The cache instance.
   pub cache: Cache,
}

/// Represents Apache-related data in the application.
#[derive(Serialize, Deserialize, Default)]
pub struct Apache {
   /// The ID of the Apache instance.
   pub id: String,
   /// The stylesheet URL.
   pub stylesheet: String,
   /// The title of the page.
   pub title: String,
   /// The description of the page.
   pub description: String,
   /// The keywords for the page.
   pub keywords: String,
}

/// Represents a collection of posts or pages in the application.
#[derive(Serialize, Default)]
pub struct Poster<T> {
   /// A list of posts or pages.
   pub posts: Vec<T>,
   /// A list of page numbers.
   pub pages: Vec<i32>,
   /// Whether there are multiple pages.
   pub has_pages: bool,
   /// Whether there is a previous page.
   pub has_prev: bool,
   /// Whether there is a next page.
   pub has_next: bool,
   /// The current page number.
   pub page: i32,
   /// The previous page number.
   pub prev_page: i32,
   /// The next page number.
   pub next_page: i32,
}

/// Represents an error in the application.
#[derive(Deserialize, Serialize, Default)]
pub struct Error {
   /// The error code.
   pub code: String,
   /// The error name.
   pub name: String,
}

/// Represents a message sent by the application.
#[derive(Deserialize, Serialize, Default)]
pub struct Message {
   /// The type of the message.
   pub r#type: String,
   /// The text content of the message.
   pub text: String,
}

/// Represents an authentication request in the application.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthRequest {
   /// The authorization code.
   pub code: String,
   /// The scope of the authorization (optional).
   pub scope: Option<String>,
   /// The CSRF token for the request.
   pub state: CsrfToken,
}

/// Represents an authorized user in the application.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AuthorizedUser {
   /// The login or name of the user.
   pub login_or_name: String,
   /// Whether the user is authenticated.
   pub authenticated: bool,
   /// Whether the user has admin privileges.
   pub admin: bool,
   /// The provider of the user's account.
   pub provider: String,
}

/// Represents a container for files in the application.
#[derive(Serialize, Default, ToSchema)]
pub struct FilesContainer {
   /// The title of the file collection.
   #[serde(rename(serialize = "Title"))]
   #[schema(rename = "Title")]
   pub title: String,
   /// A list of downloadable files.
   #[serde(rename(serialize = "Files"))]
   #[schema(rename = "Files")]
   pub files: Vec<Downloadable>,
}

/// Represents a downloadable file in the application.
#[derive(Serialize, Default, ToSchema)]
pub struct Downloadable {
   /// The title of the file.
   #[serde(rename(serialize = "Title"))]
   #[schema(rename = "Title")]
   pub title: String,
   /// The path to the file.
   #[serde(rename(serialize = "Path"))]
   #[schema(rename = "Path")]
   pub path: String,
   /// The filename of the downloadable file.
   #[serde(rename(serialize = "FileName"))]
   #[schema(rename = "FileName")]
   pub filename: String,
   /// The Blake3 hash of the file.
   #[serde(rename(serialize = "Blake3Hash"))]
   #[schema(rename = "Blake3Hash")]
   pub blake3_hash: String,
   /// The size of the downloadable file in bytes.
   #[serde(rename(serialize = "Size"))]
   #[schema(rename = "Size")]
   pub size: u64,
}

impl<T> Poster<T> {
    pub fn new(api: ApiResult<T>, page: i32) -> Self {
        let pages_count = api.pages;
        let prev_page = if page == 1 { 1 } else { page - 1 };
        let next_page = if page == pages_count {
            pages_count
        } else {
            page + 1
        };
        let pages: Vec<i32> = (1..=pages_count).collect();
        Self {
            posts: api.result,
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
    use kernel::domain::SmallPost;

    use super::*;

    #[test]
    fn poster_new_with_pages_first_page() {
        // arrange
        let api_result = ApiResult::<SmallPost> {
            result: vec![],
            pages: 2,
            page: 1,
            count: 40,
            status: "",
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
        let api_result = ApiResult::<SmallPost> {
            result: vec![],
            pages: 2,
            page: 1,
            count: 40,
            status: "",
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
        let api_result = ApiResult::<SmallPost> {
            result: vec![],
            pages: 3,
            page: 1,
            count: 60,
            status: "",
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
        let api_result = ApiResult::<SmallPost> {
            result: vec![],
            pages: 1,
            page: 1,
            count: 20,
            status: "",
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
