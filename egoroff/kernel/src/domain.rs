use chrono::{DateTime, Utc};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SmallPost {
    pub created: DateTime<Utc>,
    pub id: i64,
    pub title: String,
    pub short_text: String,
    pub markdown: bool,
}
