use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::{error::Error, fmt::Debug};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct User {
    pub created: DateTime<Utc>,
    pub email: String,
    pub name: String,
    #[serde(rename(serialize = "username"))]
    pub login: String,
    #[serde(rename(serialize = "avatarUrl"))]
    pub avatar_url: String,
    pub federated_id: String,
    pub admin: bool,
    pub verified: bool,
    pub provider: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct SmallPost {
    #[serde(rename(serialize = "Created"))]
    #[schema(rename = "Created")]
    pub created: DateTime<Utc>,
    #[schema(example = 66)]
    pub id: i64,
    #[schema(example = "Blake3", rename = "Title")]
    #[serde(rename(serialize = "Title"))]
    pub title: String,
    #[schema(example = "# About Blake3", rename = "ShortText")]
    #[serde(rename(serialize = "ShortText"))]
    pub short_text: String,
    #[serde(skip_serializing)]
    pub markdown: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Post {
    #[serde(rename(serialize = "Created", deserialize = "Created"))]
    pub created: DateTime<Utc>,
    #[serde(rename(serialize = "Modified", deserialize = "Modified"))]
    pub modified: DateTime<Utc>,
    pub id: i64,
    #[serde(rename(serialize = "Title", deserialize = "Title"))]
    pub title: String,
    #[serde(rename(serialize = "ShortText", deserialize = "ShortText"))]
    pub short_text: String,
    #[serde(rename(serialize = "Text", deserialize = "Text"))]
    pub text: String,
    #[serde(rename(serialize = "Markdown", deserialize = "Markdown"))]
    pub markdown: bool,
    #[serde(rename(serialize = "IsPublic", deserialize = "IsPublic"))]
    pub is_public: bool,
    #[serde(rename(serialize = "Tags", deserialize = "Tags"))]
    pub tags: Vec<String>,
}

impl Post {
    #[must_use]
    pub fn keywords(&self) -> String {
        self.tags.join(",")
    }
}

#[derive(Deserialize, Serialize, Default, Clone, IntoParams)]
pub struct PostsRequest {
    pub tag: Option<String>,
    pub page: Option<i32>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub include_private: Option<bool>,
}

#[derive(Deserialize, Serialize, Default, Clone, IntoParams)]
pub struct DownloadsRequest {
    pub page: Option<i32>,
}

pub struct Period {
    pub from: DateTime<Utc>,
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

#[derive(Serialize, Default)]
pub struct Archive {
    pub tags: Vec<Tag>,
    pub years: Vec<Year>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Tag {
    pub title: String,
    pub level: usize,
}

#[derive(Deserialize, Serialize, Default)]
pub struct TagAggregate {
    pub title: String,
    pub count: i32,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Year {
    pub year: i32,
    pub posts: i32,
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

#[derive(Deserialize, Serialize, Default)]
pub struct Month {
    pub month: i32,
    pub posts: i32,
}

#[derive(Serialize, Default, ToSchema)]
pub struct ApiResult<T> {
    pub result: Vec<T>,
    #[schema(example = 4)]
    pub pages: i32,
    #[schema(example = 1)]
    pub page: i32,
    #[schema(example = 68)]
    pub count: i32,
    #[schema(example = "success")]
    pub status: &'static str,
}

#[derive(Serialize, Default)]
pub struct OAuthProvider {
    pub name: String,
    pub client_id: String,
    pub secret: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

#[derive(Serialize, Default)]
pub struct Folder {
    pub bucket: String,
    pub title: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Download {
    pub id: i64,
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
