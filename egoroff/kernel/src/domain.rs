use chrono::{DateTime, Utc};
use std::fmt::{Debug, Display};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SmallPost {
    #[serde(rename(serialize = "Created"))]
    pub created: DateTime<Utc>,
    pub id: i64,
    #[serde(rename(serialize = "Title"))]
    pub title: String,
    #[serde(rename(serialize = "ShortText"))]
    pub short_text: String,
    #[serde(rename(serialize = "Markdown"))]
    pub markdown: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Post {
    #[serde(rename(deserialize = "Created"))]
    pub created: DateTime<Utc>,
    #[serde(rename(deserialize = "Modified"))]
    pub modified: DateTime<Utc>,
    pub id: i64,
    #[serde(rename(deserialize = "Title"))]
    pub title: String,
    #[serde(rename(deserialize = "ShortText"))]
    pub short_text: String,
    #[serde(rename(deserialize = "Text"))]
    pub text: String,
    #[serde(rename(deserialize = "Markdown"))]
    pub markdown: bool,
    #[serde(rename(deserialize = "IsPublic"))]
    pub is_public: bool,
    #[serde(rename(deserialize = "Tags"))]
    pub tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct PostsRequest {
    pub tag: Option<String>,
    pub page: Option<i32>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Archive {
    pub tags: Vec<Tag>,
    pub years: Vec<Year>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Tag {
    pub title: String,
    pub level: String,
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

#[derive(Deserialize, Serialize, Default)]
pub struct Month {
    pub month: i32,
    pub posts: i32,
    pub name: String,
}

#[derive(Serialize, Default)]
pub struct ApiResult {
    pub result: Vec<SmallPost>,
    pub pages: i32,
    pub page: i32,
    pub count: i32,
    pub status: String,
}

pub trait Storage {
    type Err: Debug + Display;

    fn new_database(&self) -> Result<(), Self::Err>;
    fn get_small_posts(&self, limit: i32, offset: i32, request: PostsRequest) -> Result<Vec<SmallPost>, Self::Err>;
    fn get_post(&self, id: i64) -> Result<Post, Self::Err>;
    fn upsert_post(&mut self, post: Post) -> Result<(), Self::Err>;
    fn delete_post(&mut self, id: i64) -> Result<(), Self::Err>;
    fn count_posts(&self, request: PostsRequest) -> Result<i32, Self::Err>;
    fn get_aggregate_tags(&self) -> Result<Vec<TagAggregate>, Self::Err>;
    fn get_posts_create_dates(&self) -> Result<Vec<DateTime<Utc>>, Self::Err>;
}
