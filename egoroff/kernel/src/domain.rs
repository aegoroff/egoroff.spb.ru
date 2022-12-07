use chrono::{DateTime, Utc};

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
