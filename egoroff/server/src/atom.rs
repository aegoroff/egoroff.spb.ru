use kernel::domain::SmallPost;
use serde::Serialize;
use serde_xml_rs::to_string;

#[derive(Debug, Serialize)]
#[serde(rename = "feed")]
pub struct Feed {
    #[serde(rename = "$value")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
#[serde(rename = "entry")]
pub struct Entry {
    #[serde(rename = "$value")]
    pub title: Title,
    #[serde(rename = "$value")]
    pub id: String,
    #[serde(rename = "$value")]
    pub updated: String,
    #[serde(rename = "$value")]
    pub published: String,
    #[serde(rename = "$value")]
    pub content: String,
    #[serde(rename = "$value")]
    pub author: String,
}

#[derive(Debug, Serialize)]
#[serde(rename = "title")]
pub struct Title {
    #[serde(rename = "$value")]
    pub value: String,
    #[serde(rename = "@type")]
    pub attr_type: String,
}

pub fn from_small_posts(posts: Vec<SmallPost>) -> String {
    let entries = posts
        .iter()
        .map(|p| Entry {
            title: Title {
                value: p.title.clone(),
                attr_type: "text".to_string(),
            },
            id: p.id.to_string(),
            updated: p.created.to_rfc3339(),
            published: p.created.to_rfc3339(),
            content: p.short_text.clone(),
            author: "Alexander Egorov".to_string(),
        })
        .collect();
    let feed = Feed { entries };
    to_string(&feed).unwrap()
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, Utc};

    use super::*;

    #[test]
    fn from_small_posts_tests() {
        // arrange
        let dt1 = NaiveDate::from_ymd_opt(2015, 2, 2)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap();
        let dt1 = DateTime::<Utc>::from_local(dt1, Utc);
        let p1 = SmallPost {
            created: dt1,
            id: 1,
            title: "title 1".to_string(),
            short_text: "txt 1".to_string(),
            markdown: true,
        };

        let dt2 = NaiveDate::from_ymd_opt(2015, 2, 2)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap();
        let dt2 = DateTime::<Utc>::from_local(dt2, Utc);
        let p2 = SmallPost {
            created: dt2,
            id: 2,
            title: "title 2".to_string(),
            short_text: "txt 2".to_string(),
            markdown: true,
        };
        let posts = vec![p1, p2];

        // act
        let actual = from_small_posts(posts);

        // assert
        assert!(!actual.is_empty())
    }
}
