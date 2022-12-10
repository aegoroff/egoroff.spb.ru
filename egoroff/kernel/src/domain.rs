use chrono::{DateTime, Utc};
use std::fmt::{Debug, Display};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SmallPost {
    pub created: DateTime<Utc>,
    pub id: i64,
    pub title: String,
    pub short_text: String,
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

pub trait Storage {
    type Err : Debug + Display;

    fn new_database(&self) -> Result<(), Self::Err>;
    fn get_small_posts(&self, limit: i32, offset: i32) -> Result<Vec<SmallPost>, Self::Err>;
    fn get_post(&self, id: i64) -> Result<Post, Self::Err>;
    fn upsert_post(&mut self, post: Post) -> Result<(), Self::Err>;
    fn delete_post(&mut self, id: i64) -> Result<(), Self::Err>;
    fn count_posts(&self) -> Result<i32, Self::Err>;
}