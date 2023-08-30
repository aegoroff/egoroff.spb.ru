use anyhow::Result;
use chrono::SecondsFormat;
use kernel::{domain::SmallPost, xml::Builder};

const FEED_ELT: &str = "feed";
const LINK_ELT: &str = "link";
const ENTRY_ELT: &str = "entry";
const AUTHOR_ELT: &str = "author";

pub fn from_small_posts(posts: Vec<SmallPost>) -> Result<String> {
    let mut builder = Builder::new();

    builder.write_attributed_start_tag(FEED_ELT, vec![("xmlns", "http://www.w3.org/2005/Atom")])?;

    builder.write_attributed_element("title", "egoroff.spb.ru feed", vec![("type", "text")])?;

    builder.write_element("id", "https://www.egoroff.spb.ru/blog/recent.atom")?;

    let updated = posts[0].created.to_rfc3339_opts(SecondsFormat::Secs, true);
    builder.write_element("updated", &updated)?;

    builder
        .write_empty_attributed_element(LINK_ELT, vec![("href", "https://www.egoroff.spb.ru/")])?;

    builder.write_empty_attributed_element(
        LINK_ELT,
        vec![
            ("href", "https://www.egoroff.spb.ru/blog/recent.atom"),
            ("rel", "self"),
        ],
    )?;

    for post in posts {
        builder.write_attributed_start_tag(
            ENTRY_ELT,
            vec![("xml:base", "https://www.egoroff.spb.ru/blog/recent.atom")],
        )?;

        builder.write_attributed_element("title", &post.title, vec![("type", "text")])?;

        builder.write_element(
            "id",
            &format!("https://www.egoroff.spb.ru/blog/{}.html", post.id),
        )?;

        let updated = post.created.to_rfc3339_opts(SecondsFormat::Secs, true);
        builder.write_element("updated", &updated)?;

        builder.write_element("published", &updated)?;

        builder.write_attributed_element("content", &post.short_text, vec![("type", "html")])?;

        builder.write_start_tag(AUTHOR_ELT)?;

        builder.write_element("name", "Alexander Egorov")?;

        builder.write_end_tag(AUTHOR_ELT)?;

        builder.write_end_tag(ENTRY_ELT)?;
    }

    builder.write_end_tag(FEED_ELT)?;

    builder.to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use chrono::{NaiveDate, Utc};

    use super::*;

    #[test]
    fn from_small_posts_tests() {
        // arrange
        let dt1 = NaiveDate::from_ymd_opt(2015, 2, 2)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .latest()
            .unwrap();
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
            .unwrap()
            .and_local_timezone(Utc)
            .latest()
            .unwrap();
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
        assert!(actual.is_ok());
        let result = actual.unwrap();
        assert!(!result.is_empty());
    }
}
