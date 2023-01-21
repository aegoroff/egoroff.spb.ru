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
    pub created: DateTime<Utc>,
    #[schema(example = 66)]
    pub id: i64,
    #[schema(example = "Blake3")]
    #[serde(rename(serialize = "Title"))]
    pub title: String,
    #[schema(example = "# About Blake3")]
    #[serde(rename(serialize = "ShortText"))]
    pub short_text: String,
    #[serde(rename(serialize = "Markdown"))]
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

pub struct Period {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

impl PostsRequest {
    pub fn as_query_period(&self) -> Option<Period> {
        let year = self.year.unwrap_or(0);
        let month = self.month.unwrap_or(0);
        if year > 0 {
            let m: u32 = if month > 0 { month as u32 } else { 1 };
            let from_dt = NaiveDate::from_ymd_opt(year, m, 1)?.and_hms_opt(0, 0, 0)?;
            let from_dt = DateTime::<Utc>::from_local(from_dt, Utc);

            let m: u32 = if month > 0 { month as u32 } else { 12 };
            let d = Self::last_day_of_month(year, m)?;
            let to_dt = NaiveDate::from_ymd_opt(year, m, d)?.and_hms_opt(23, 59, 59)?;
            let to_dt = DateTime::<Utc>::from_local(to_dt, Utc);
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

#[derive(Deserialize, Serialize, Default)]
pub struct Archive {
    pub tags: Vec<Tag>,
    #[serde(borrow = "'static")]
    pub years: Vec<Year<'static>>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Tag {
    pub title: String,
    pub level: &'static str,
}

#[derive(Deserialize, Serialize, Default)]
pub struct TagAggregate {
    pub title: String,
    pub count: i32,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Year<'a> {
    pub year: i32,
    pub posts: i32,
    #[serde(borrow = "'a")]
    pub months: Vec<Month<'a>>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Month<'a> {
    pub month: i32,
    pub posts: i32,
    pub name: &'a str,
}

#[derive(Serialize, Default, ToSchema)]
#[aliases(SmallPosts = ApiResult<SmallPost>)]
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::{test_context, TestContext};

    #[test_context(PostsRequestContext)]
    #[test]
    fn as_query_period_single_first_month(ctx: &mut PostsRequestContext) {
        // arrange
        ctx.request.year = Some(2022);
        ctx.request.month = Some(1);

        // act
        let period = ctx.request.as_query_period();

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

    #[test_context(PostsRequestContext)]
    #[test]
    fn as_query_period_single_last_month(ctx: &mut PostsRequestContext) {
        // arrange
        ctx.request.year = Some(2022);
        ctx.request.month = Some(12);

        // act
        let period = ctx.request.as_query_period();

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

    #[test_context(PostsRequestContext)]
    #[test]
    fn as_query_period_single_year(ctx: &mut PostsRequestContext) {
        // arrange
        ctx.request.year = Some(2022);

        // act
        let period = ctx.request.as_query_period();

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

    #[test_context(PostsRequestContext)]
    #[test]
    fn as_query_period_no_period(ctx: &mut PostsRequestContext) {
        // arrange

        // act
        let period = ctx.request.as_query_period();

        // assert
        assert!(period.is_none());
    }

    #[test_context(PostContext)]
    #[test]
    fn keywords_notemptytags_stringasexpected(ctx: &mut PostContext) {
        // arrange
        ctx.post.tags = vec!["a".to_string(), "b".to_string()];

        // act
        let actual = ctx.post.keywords();

        // assert
        assert_eq!("a,b", actual);
    }

    #[test_context(PostContext)]
    #[test]
    fn keywords_onetag_stringasexpected(ctx: &mut PostContext) {
        // arrange
        ctx.post.tags = vec!["a".to_string()];

        // act
        let actual = ctx.post.keywords();

        // assert
        assert_eq!("a", actual);
    }

    #[test_context(PostContext)]
    #[test]
    fn keywords_notags_stringasexpected(ctx: &mut PostContext) {
        // arrange

        // act
        let actual = ctx.post.keywords();

        // assert
        assert!(actual.is_empty());
    }

    struct PostContext {
        post: Post,
    }

    struct PostsRequestContext {
        request: PostsRequest,
    }

    impl TestContext for PostContext {
        fn setup() -> PostContext {
            PostContext {
                post: Post {
                    ..Default::default()
                },
            }
        }

        fn teardown(self) {
            // Perform any teardown you wish.
        }
    }

    impl TestContext for PostsRequestContext {
        fn setup() -> PostsRequestContext {
            PostsRequestContext {
                request: PostsRequest {
                    ..Default::default()
                },
            }
        }

        fn teardown(self) {
            // Perform any teardown you wish.
        }
    }
}
