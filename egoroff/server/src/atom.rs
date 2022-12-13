use std::io::Cursor;

use anyhow::Result;
use axum::{
    body::{Bytes, Full},
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
};
use chrono::SecondsFormat;
use kernel::domain::SmallPost;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Writer,
};

const FEED_ELT: &str = "feed";
const LINK_ELT: &str = "link";
const ENTRY_ELT: &str = "entry";
const AUTHOR_ELT: &str = "author";

/// An XML response.
///
/// Will automatically get `Content-Type: text/xml`.
#[derive(Clone, Copy, Debug)]
pub struct Xml<T>(pub T);

impl<T> IntoResponse for Xml<T>
where
    T: Into<Full<Bytes>>,
{
    fn into_response(self) -> Response {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/xml; charset=utf-8"),
            )],
            self.0.into(),
        )
            .into_response()
    }
}

impl<T> From<T> for Xml<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

pub fn from_small_posts(posts: Vec<SmallPost>) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    write_attributed_start_tag(
        &mut writer,
        FEED_ELT,
        vec![("xmlns", "http://www.w3.org/2005/Atom")],
    )?;

    write_attributed_element(
        &mut writer,
        "title",
        "egoroff.spb.ru feed",
        vec![("type", "text")],
    )?;

    write_element(
        &mut writer,
        "id",
        "https://www.egoroff.spb.ru/blog/recent.atom",
    )?;

    let updated = posts[0].created.to_rfc3339_opts(SecondsFormat::Secs, true);
    write_element(&mut writer, "updated", &updated)?;

    write_empty_attributed_element(
        &mut writer,
        LINK_ELT,
        vec![("href", "https://www.egoroff.spb.ru/")],
    )?;

    write_empty_attributed_element(
        &mut writer,
        LINK_ELT,
        vec![
            ("href", "https://www.egoroff.spb.ru/blog/recent.atom"),
            ("rel", "self"),
        ],
    )?;

    for post in posts {
        write_attributed_start_tag(
            &mut writer,
            ENTRY_ELT,
            vec![("xml:base", "https://www.egoroff.spb.ru/blog/recent.atom")],
        )?;

        write_attributed_element(&mut writer, "title", &post.title, vec![("type", "text")])?;

        write_element(
            &mut writer,
            "id",
            &format!("https://www.egoroff.spb.ru/blog/{}.html", post.id),
        )?;

        let updated = post.created.to_rfc3339_opts(SecondsFormat::Secs, true);
        write_element(&mut writer, "updated", &updated)?;

        write_element(&mut writer, "published", &updated)?;

        write_attributed_element(
            &mut writer,
            "content",
            &post.short_text,
            vec![("type", "html")],
        )?;

        write_start_tag(&mut writer, AUTHOR_ELT)?;

        write_element(&mut writer, "name", "Alexander Egorov")?;

        write_end_tag(&mut writer, AUTHOR_ELT)?;

        write_end_tag(&mut writer, ENTRY_ELT)?;
    }

    write_end_tag(&mut writer, FEED_ELT)?;

    let mut result = writer.into_inner().into_inner();

    let mut xml: Vec<u8> = Vec::from("<?xml version=\"1.0\"?>");
    xml.append(&mut result);
    let result = String::from_utf8(xml)?;
    Ok(result)
}

fn write_text(writer: &mut Writer<Cursor<Vec<u8>>>, txt: &str) -> Result<(), anyhow::Error> {
    let txt = BytesText::new(txt);
    writer.write_event(Event::Text(txt))?;
    Ok(())
}

fn write_start_tag(writer: &mut Writer<Cursor<Vec<u8>>>, elt: &str) -> Result<(), anyhow::Error> {
    let start_tag = BytesStart::new(elt);
    writer.write_event(Event::Start(start_tag))?;
    Ok(())
}

fn write_attributed_start_tag(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    elt: &str,
    attributes: Vec<(&str, &str)>,
) -> Result<(), anyhow::Error> {
    let mut start_tag = BytesStart::new(elt);
    for (attr, val) in attributes {
        start_tag.push_attribute((attr, val));
    }
    writer.write_event(Event::Start(start_tag))?;
    Ok(())
}

fn write_end_tag(writer: &mut Writer<Cursor<Vec<u8>>>, elt: &str) -> Result<(), anyhow::Error> {
    writer.write_event(Event::End(BytesEnd::new(elt)))?;
    Ok(())
}

fn write_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    elt: &str,
    txt: &str,
) -> Result<(), anyhow::Error> {
    write_start_tag(writer, elt)?;

    write_text(writer, txt)?;

    write_end_tag(writer, elt)?;
    Ok(())
}

fn write_attributed_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    elt: &str,
    txt: &str,
    attributes: Vec<(&str, &str)>,
) -> Result<(), anyhow::Error> {
    write_attributed_start_tag(writer, elt, attributes)?;

    write_text(writer, txt)?;

    write_end_tag(writer, elt)?;
    Ok(())
}

fn write_empty_attributed_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    elt: &str,
    attributes: Vec<(&str, &str)>,
) -> Result<(), anyhow::Error> {
    write_attributed_start_tag(writer, elt, attributes)?;

    write_end_tag(writer, elt)?;
    Ok(())
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
        assert!(actual.is_ok());
        let result = actual.unwrap();
        assert!(!result.is_empty());
    }
}
