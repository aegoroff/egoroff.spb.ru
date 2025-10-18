use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::{error::Error, fmt::Debug};
use utoipa::{IntoParams, ToSchema};

/// Represents a user in the system.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct User {
    /// The timestamp when the user was created.
    pub created: DateTime<Utc>,
    /// The email address associated with the user.
    pub email: String,
    /// The display name of the user.
    pub name: String,
    /// The username used to log in to the system.
    #[serde(rename(serialize = "username"))]
    pub login: String,
    /// The URL of the user's avatar image.
    #[serde(rename(serialize = "avatarUrl"))]
    pub avatar_url: String,
    /// The federated ID of the user (e.g., from an external authentication provider).
    pub federated_id: String,
    /// A boolean indicating whether the user is an administrator.
    pub admin: bool,
    /// A boolean indicating whether the user's email address has been verified.
    pub verified: bool,
    /// The name of the authentication provider used to create the user account.
    pub provider: String,
}

/// Represents a small post (e.g., for the home page).
#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct SmallPost {
    /// The timestamp when the post was created.
    #[serde(rename(serialize = "Created"))]
    #[schema(rename = "Created")]
    pub created: DateTime<Utc>,
    /// The unique ID of the post.
    #[schema(example = 66)]
    pub id: i64,
    /// The title of the post.
    #[schema(example = "Blake3", rename = "Title")]
    #[serde(rename(serialize = "Title"))]
    pub title: String,
    /// A short summary or teaser text for the post.
    #[schema(example = "# About Blake3", rename = "ShortText")]
    #[serde(rename(serialize = "ShortText"))]
    pub short_text: String,
    /// A boolean indicating whether the post content is in Markdown format (not serialized).
    #[serde(skip_serializing)]
    pub markdown: bool,
}

/// Represents a regular post.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Post {
    /// The timestamp when the post was created.
    #[serde(rename(serialize = "Created", deserialize = "Created"))]
    pub created: DateTime<Utc>,
    /// The timestamp when the post was last modified.
    #[serde(rename(serialize = "Modified", deserialize = "Modified"))]
    pub modified: DateTime<Utc>,
    /// The unique ID of the post.
    pub id: i64,
    /// The title of the post.
    #[serde(rename(serialize = "Title", deserialize = "Title"))]
    pub title: String,
    /// A short summary or teaser text for the post.
    #[serde(rename(serialize = "ShortText", deserialize = "ShortText"))]
    pub short_text: String,
    /// The full content of the post in Markdown format.
    #[serde(rename(serialize = "Text", deserialize = "Text"))]
    pub text: String,
    /// A boolean indicating whether the post content is in Markdown format.
    #[serde(rename(serialize = "Markdown", deserialize = "Markdown"))]
    pub markdown: bool,
    /// A boolean indicating whether the post is publicly visible.
    #[serde(rename(serialize = "IsPublic", deserialize = "IsPublic"))]
    pub is_public: bool,
    /// A list of tags associated with the post.
    #[serde(rename(serialize = "Tags", deserialize = "Tags"))]
    pub tags: Vec<String>,
}

impl Post {
    #[must_use]
    pub fn keywords(&self) -> String {
        self.tags.join(",")
    }
}

/// Request for retrieving posts.
///
/// This struct holds parameters for filtering posts based on tag, page number,
/// year, month, and whether to include private posts.
#[derive(Deserialize, Serialize, Default, Clone, IntoParams)]
pub struct PostsRequest {
    /// The tag to filter posts by (optional).
    pub tag: Option<String>,
    /// The page number of posts to retrieve (optional).
    pub page: Option<i32>,
    /// The year to filter posts by (optional).
    pub year: Option<i32>,
    /// The month to filter posts by (optional).
    pub month: Option<i32>,
    /// Whether to include private posts in the result (optional).
    pub include_private: Option<bool>,
}

/// Request for retrieving downloads.
///
/// This struct holds a single parameter for specifying the page number of
/// downloads to retrieve.
#[derive(Deserialize, Serialize, Default, Clone, IntoParams)]
pub struct DownloadsRequest {
    /// The page number of downloads to retrieve (optional).
    pub page: Option<i32>,
}

/// A time period with start and end dates in UTC.
#[derive(Deserialize, Serialize, Default)]
pub struct Period {
    /// The start date of the period (inclusive) in UTC.
    pub from: DateTime<Utc>,
    /// The end date of the period (inclusive) in UTC.
    pub to: DateTime<Utc>,
}

impl PostsRequest {
    #[must_use]
    pub fn as_query_period(&self) -> Option<Period> {
        let year = self.year.unwrap_or(0);
        let month = self.month.unwrap_or(0);
        if year > 0 {
            let m: u32 = if month > 0 { month as u32 } else { 1 };
            let from_dt = NaiveDate::from_ymd_opt(year, m, 1)?
                .and_hms_opt(0, 0, 0)?
                .and_local_timezone(Utc)
                .latest()?;

            let m: u32 = if month > 0 { month as u32 } else { 12 };
            let d = Self::last_day_of_month(year, m)?;
            let to_dt = NaiveDate::from_ymd_opt(year, m, d)?
                .and_hms_opt(23, 59, 59)?
                .and_local_timezone(Utc)
                .latest()?;
            Some(Period {
                from: from_dt,
                to: to_dt,
            })
        } else {
            None
        }
    }

    fn last_day_of_month(year: i32, month: u32) -> Option<u32> {
        let first_day_of_next_year = NaiveDate::from_ymd_opt(year + 1, 1, 1)?;
        let d = NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap_or(first_day_of_next_year)
            .pred_opt()?
            .day();
        Some(d)
    }
}

/// Represents a collection of archives.
#[derive(Serialize, Default)]
pub struct Archive {
    /// A list of tags associated with this archive.
    pub tags: Vec<Tag>,
    /// A list of years for this archive.
    pub years: Vec<Year>,
}

/// Represents a single tag.
#[derive(Deserialize, Serialize, Default)]
pub struct Tag {
    /// The title of the tag.
    pub title: String,
    /// The level of importance or relevance of the tag (1-10).
    pub level: usize,
}

/// Represents an aggregate count of tags.
#[derive(Deserialize, Serialize, Default)]
pub struct TagAggregate {
    /// The title of the aggregated tag.
    pub title: String,
    /// The total count of posts associated with this tag.
    pub count: i32,
}

/// Represents a single year in the archive.
#[derive(Deserialize, Serialize, Default)]
pub struct Year {
    /// The year in question (e.g. 2022).
    pub year: i32,
    /// The number of posts published during this year.
    pub posts: i32,
    /// A list of months within this year.
    pub months: Vec<Month>,
}

impl Year {
    #[must_use]
    pub fn new(year: i32) -> Self {
        Self {
            year,
            posts: 0,
            months: Vec::with_capacity(12),
        }
    }

    pub fn append_month(&mut self, m: Month) {
        self.posts += m.posts;
        self.months.push(m);
    }
}

/// Represents a month with its corresponding number of posts.
///
/// This struct is used for deserialization and serialization purposes only.
#[derive(Deserialize, Serialize, Default)]
pub struct Month {
    /// The month number (1-12).
    pub month: i32,
    /// The total number of posts in the month.
    pub posts: i32,
}

/// Represents an API result with a list of items and pagination information.
///
/// This struct is used for serialization and deserialization purposes only.
#[derive(Serialize, Default, ToSchema)]
pub struct ApiResult<T> {
    /// A vector of items returned by the API.
    pub result: Vec<T>,
    /// The total number of pages in the result.
    #[schema(example = 4)]
    pub pages: i32,
    /// The current page number.
    #[schema(example = 1)]
    pub page: i32,
    /// The total count of items returned by the API.
    #[schema(example = 68)]
    pub count: i32,
    /// A status message indicating whether the operation was successful.
    #[schema(example = "success")]
    pub status: &'static str,
}

/// Represents an OAuth provider with its authentication settings.
///
/// This struct is used for serialization purposes only.
#[derive(Serialize, Default)]
pub struct OAuthProvider {
    /// The name of the OAuth provider (e.g. Google, Facebook).
    pub name: String,
    /// The client ID of the OAuth provider.
    pub client_id: String,
    /// The client secret of the OAuth provider.
    pub secret: String,
    /// The redirect URL for the OAuth provider's authentication flow.
    pub redirect_url: String,
    /// A list of scopes required by the OAuth provider.
    pub scopes: Vec<String>,
}

/// Represents a folder with its bucket and title information.
///
/// This struct is used for serialization purposes only.
#[derive(Serialize, Default)]
pub struct Folder {
    /// The name of the bucket containing the folder.
    pub bucket: String,
    /// The title of the folder.
    pub title: String,
}

/// Represents a downloadable file with its ID and title information.
///
/// This struct is used for deserialization and serialization purposes only.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Download {
    /// The unique ID of the download.
    pub id: i64,
    /// The title of the downloadable file.
    pub title: String,
}

pub trait Storage {
    type Err: Sync + Send + Error + 'static;

    fn new_database(&self) -> Result<(), Self::Err>;
    fn get_small_posts(
        &self,
        limit: i32,
        offset: i32,
        request: PostsRequest,
    ) -> Result<Vec<SmallPost>, Self::Err>;
    fn get_posts(&self, limit: i32, offset: i32) -> Result<Vec<Post>, Self::Err>;
    fn get_post(&self, id: i64) -> Result<Post, Self::Err>;
    fn get_new_post_id(&self, id: i64) -> Result<i64, Self::Err>;
    fn upsert_post(&mut self, post: Post) -> Result<(), Self::Err>;
    fn next_post_id(&mut self) -> Result<i64, Self::Err>;
    fn delete_post(&mut self, id: i64) -> Result<usize, Self::Err>;
    fn count_posts(&self, request: PostsRequest) -> Result<i32, Self::Err>;
    fn get_aggregate_tags(&self) -> Result<Vec<TagAggregate>, Self::Err>;
    fn get_posts_create_dates(&self) -> Result<Vec<DateTime<Utc>>, Self::Err>;
    fn get_posts_ids(&self) -> Result<Vec<i64>, Self::Err>;
    fn get_oauth_provider(&self, name: &str) -> Result<OAuthProvider, Self::Err>;
    fn get_user(&self, federated_id: &str, provider: &str) -> Result<User, Self::Err>;
    fn upsert_user(&mut self, user: &User) -> Result<(), Self::Err>;
    fn get_folders(&self) -> Result<Vec<Folder>, Self::Err>;
    fn get_download(&self, id: i64) -> Result<Download, Self::Err>;
    fn upsert_download(&mut self, download: Download) -> Result<(), Self::Err>;
    fn delete_download(&mut self, id: i64) -> Result<usize, Self::Err>;
    fn get_downloads(&self, limit: i32, offset: i32) -> Result<Vec<Download>, Self::Err>;
    fn count_downloads(&self) -> Result<i32, Self::Err>;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::{fixture, rstest};

    #[rstest]
    fn as_query_period_single_first_month(mut posts_req: PostsRequest) {
        // arrange
        posts_req.year = Some(2022);
        posts_req.month = Some(1);

        // act
        let period = posts_req.as_query_period();

        // assert
        assert!(period.is_some());
        let period = period.unwrap();
        assert_eq!(period.from.year(), 2022);
        assert_eq!(period.from.month(), 1);
        assert_eq!(period.from.day(), 1);
        assert_eq!(period.to.year(), 2022);
        assert_eq!(period.to.month(), 1);
        assert_eq!(period.to.day(), 31);
    }

    #[rstest]
    fn as_query_period_single_last_month(mut posts_req: PostsRequest) {
        // arrange
        posts_req.year = Some(2022);
        posts_req.month = Some(12);

        // act
        let period = posts_req.as_query_period();

        // assert
        assert!(period.is_some());
        let period = period.unwrap();
        assert_eq!(period.from.year(), 2022);
        assert_eq!(period.from.month(), 12);
        assert_eq!(period.from.day(), 1);
        assert_eq!(period.to.year(), 2022);
        assert_eq!(period.to.month(), 12);
        assert_eq!(period.to.day(), 31);
    }

    #[rstest]
    fn as_query_period_single_year(mut posts_req: PostsRequest) {
        // arrange
        posts_req.year = Some(2022);

        // act
        let period = posts_req.as_query_period();

        // assert
        assert!(period.is_some());
        let period = period.unwrap();
        assert_eq!(period.from.year(), 2022);
        assert_eq!(period.from.month(), 1);
        assert_eq!(period.from.day(), 1);
        assert_eq!(period.to.year(), 2022);
        assert_eq!(period.to.month(), 12);
        assert_eq!(period.to.day(), 31);
    }

    #[rstest]
    fn as_query_period_no_period(posts_req: PostsRequest) {
        // arrange

        // act
        let period = posts_req.as_query_period();

        // assert
        assert!(period.is_none());
    }

    #[rstest]
    fn keywords_notemptytags_stringasexpected(mut post: Post) {
        // arrange
        post.tags = vec!["a".to_string(), "b".to_string()];

        // act
        let actual = post.keywords();

        // assert
        assert_eq!("a,b", actual);
    }

    #[rstest]
    fn keywords_onetag_stringasexpected(mut post: Post) {
        // arrange
        post.tags = vec!["a".to_string()];

        // act
        let actual = post.keywords();

        // assert
        assert_eq!("a", actual);
    }

    #[rstest]
    fn keywords_notags_stringasexpected(post: Post) {
        // arrange

        // act
        let actual = post.keywords();

        // assert
        assert!(actual.is_empty());
    }

    #[fixture]
    fn post() -> Post {
        Post {
            ..Default::default()
        }
    }

    #[fixture]
    fn posts_req() -> PostsRequest {
        PostsRequest {
            ..Default::default()
        }
    }
}
