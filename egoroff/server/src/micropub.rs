use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::form_urlencoded::parse;
use utoipa::ToSchema;

#[derive(Serialize, Default, ToSchema)]
pub struct MicropubConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<Vec<String>>,
    #[serde(
        rename(serialize = "media-endpoint"),
        skip_serializing_if = "Option::is_none"
    )]
    pub media_endpoint: Option<String>,
    #[serde(
        rename(serialize = "syndicate-to"),
        skip_serializing_if = "Option::is_none"
    )]
    pub syndicate_to: Option<Vec<SyndicateTo>>,
}

#[derive(Serialize, ToSchema)]
pub struct SyndicateTo {
    pub uid: String,
    pub name: String,
}

#[derive(Debug, Error, ToSchema)]
pub enum MicropubFormError {
    #[error("Required field '{0}' is missing.")]
    MissingField(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum MicropubPropertyValue {
    Value(String),
    Values(Vec<String>),
    Map(HashMap<String, MicropubPropertyValue>),
    VecMap(Vec<HashMap<String, MicropubPropertyValue>>),
    ValueVec(Vec<MicropubPropertyValue>),
}

#[derive(Clone, Debug, Deserialize)]
pub struct MicropubProperties(HashMap<String, MicropubPropertyValue>);

#[derive(Debug, Deserialize)]
pub struct MicropubJSONCreate {
    #[serde(rename = "type")]
    entry_type: Vec<String>,
    properties: MicropubProperties,
}

// An earlier take on this was an enum with Url and Props variants
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Photo {
    url: String,
    alt: Option<String>,
}

// TODO:
// - quill appears to include 'published' and 'created' properties
// - food entries seem... complex. See food entry test case below
//   e.g. a 'drank' property may have a whole sub type/properties object...
//   I'd really like to support recording this for e.g. tea blogging but this might require a
//   larger refactor.
// - bookmark might have a bookmark-of property (possibly more likely to be a form encoded than
//   json encoded entry
// - review types (https://quill.p3k.io/review)
//   quill doesn't appeart to include categories in the form but that would be nice to support
//   adding a test case below, commented out
#[derive(Debug, Deserialize)]
pub struct MicropubFormBuilder {
    access_token: Option<String>,
    h: Option<String>,
    content: Option<String>,
    content_type: Option<String>,
    category: Option<Vec<String>>,
    name: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    slug: Option<String>,
    bookmark_of: Option<String>,
    photos: Option<Vec<Photo>>,
}

impl MicropubFormBuilder {
    pub fn new() -> Self {
        Self {
            access_token: None,
            h: None,
            content: None,
            content_type: None,
            category: None,
            name: None,
            created_at: None,
            updated_at: None,
            slug: None,
            bookmark_of: None,
            photos: None,
        }
    }

    pub fn from_json(json_bytes: &[u8]) -> Result<Self> {
        let json_create: MicropubJSONCreate = serde_json::from_slice(json_bytes)?;
        let mut builder = MicropubFormBuilder::new();

        if let Some(entry_type) = json_create.entry_type.first() {
            // Normalizes h-entry or h-food into entry and food
            builder.set_h(entry_type.strip_prefix("h-").unwrap_or(entry_type).into());
        }

        for (p, v) in json_create.properties.0 {
            match p.as_str() {
                "content" | "content[html]" => builder.handle_content(v),
                "name" => builder.handle_name(v),
                "category" => builder.handle_category(v),
                "published" => builder.handle_published(v),
                "mp-slug" => builder.handle_slug(v),
                "bookmark-of" => builder.handle_bookmark(v),
                "photo" => builder.handle_photo(v),
                _ => {}
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<MicropubForm, MicropubFormError> {
        Ok(MicropubForm {
            access_token: self.access_token,
            h: self
                .h
                .ok_or_else(|| MicropubFormError::MissingField("h".into()))?,
            content: self
                .content
                .ok_or_else(|| MicropubFormError::MissingField("content".into()))?,
            content_type: self.content_type,
            category: self.category.unwrap_or_default(),
            name: self.name,
            created_at: self.created_at,
            updated_at: self.updated_at,
            slug: self.slug,
            bookmark_of: self.bookmark_of,
            photos: self.photos,
        })
    }

    fn handle_content(&mut self, val: MicropubPropertyValue) {
        match val {
            MicropubPropertyValue::Values(vals) => {
                vals.first()
                    .iter()
                    .for_each(|s| self.set_content((**s).clone()));
            }
            MicropubPropertyValue::VecMap(vecmap) => {
                // we may get {"content": [{"html": "blah"}]}
                // see test case
                vecmap.first().iter().for_each(|map| {
                    if let Some(MicropubPropertyValue::Value(content)) = map.get("html") {
                        self.set_content_type("html".into());
                        self.set_content(content.clone());
                    } else if let Some(MicropubPropertyValue::Value(content)) = map.get("markdown")
                    {
                        self.set_content_type("markdown".into());
                        self.set_content(content.clone());
                    }
                });
            }
            MicropubPropertyValue::Value(val) => {
                self.set_content(val);
            }
            _ => tracing::error!("unexpected content type"),
        }
    }

    fn handle_name(&mut self, val: MicropubPropertyValue) {
        if let MicropubPropertyValue::Values(vals) = val {
            vals.first()
                .iter()
                .for_each(|s| self.set_name((**s).clone()));
        } else {
            tracing::error!("unexpected name type");
        };
    }

    fn handle_category(&mut self, val: MicropubPropertyValue) {
        match val {
            MicropubPropertyValue::Value(c) => {
                self.add_category(c);
            }
            MicropubPropertyValue::Values(cs) => {
                cs.iter().for_each(|c| self.add_category(c.clone()));
            }
            _ => tracing::error!("unexpected category type"),
        }
    }

    fn handle_published(&mut self, val: MicropubPropertyValue) {
        if let MicropubPropertyValue::Values(dates) = val {
            if dates.len() != 1 {
                tracing::error!("unexpected published dates length");
                return;
            }
            self.set_created_at(dates[0].clone());
        } else {
            tracing::error!("unexpected published type");
        }
    }

    fn handle_slug(&mut self, val: MicropubPropertyValue) {
        match val {
            MicropubPropertyValue::Values(slugs) => {
                if slugs.len() != 1 {
                    tracing::error!("unexpected slugs length");
                    return;
                }
                self.set_slug(slugs[0].clone());
            }
            MicropubPropertyValue::Value(slug) => self.set_slug(slug),
            _ => tracing::error!("unexpected slug type"),
        }
    }

    fn handle_bookmark(&mut self, val: MicropubPropertyValue) {
        match val {
            MicropubPropertyValue::Values(mut bookmark_urls) => {
                if bookmark_urls.len() != 1 {
                    // TODO log
                    return;
                }
                // TODO is there a different entry type we should set here? Should an extra
                // post type column be added? Seems others (and clients) still set
                // entry_type as h-entry so maybe the latter?
                self.set_bookmark_of(
                    bookmark_urls
                        .pop()
                        .expect("bookmark_urls len was checked as 1"),
                );
            }
            _ => eprintln!("unexpected bookmark_of property type"),
        }
    }

    fn set_access_token(&mut self, val: String) {
        self.access_token = Some(val);
    }

    fn set_h(&mut self, val: String) {
        self.h = Some(val);
    }

    fn set_content(&mut self, val: String) {
        self.content = Some(val);
    }

    fn set_content_type(&mut self, val: String) {
        self.content_type = Some(val);
    }

    fn add_category(&mut self, val: String) {
        if self.category.is_none() {
            self.category = Some(vec![]);
        }

        if let Some(categories) = self.category.as_mut() {
            categories.push(val);
        }
    }

    fn set_name(&mut self, val: String) {
        self.name = Some(val);
    }

    fn set_created_at(&mut self, val: String) {
        self.created_at = Some(val);
    }

    fn set_slug(&mut self, val: String) {
        self.slug = Some(val);
    }

    fn set_bookmark_of(&mut self, val: String) {
        self.bookmark_of = Some(val);
    }

    fn add_photo(&mut self, val: Photo) {
        if self.photos.is_none() {
            self.photos = Some(vec![]);
        }

        if let Some(photos) = self.photos.as_mut() {
            photos.push(val);
        }
    }

    fn handle_photo(&mut self, props: MicropubPropertyValue) {
        match props {
            MicropubPropertyValue::Value(photo_url) => {
                self.add_photo(Photo {
                    url: photo_url,
                    alt: None,
                });
            }
            MicropubPropertyValue::Values(mut photo_urls) => {
                photo_urls.drain(..).for_each(|photo_url| {
                    self.add_photo(Photo {
                        url: photo_url,
                        alt: None,
                    });
                });
            }
            MicropubPropertyValue::Map(mut props) => {
                if let Some(MicropubPropertyValue::Value(url)) = props.remove("value") {
                    let alt = match props.remove("alt") {
                        Some(MicropubPropertyValue::Value(alt)) => Some(alt),
                        _ => None,
                    };
                    let photo = Photo { url, alt };
                    self.add_photo(photo);
                }
            }
            MicropubPropertyValue::VecMap(mut props_vec) => {
                for mut props in props_vec.drain(..) {
                    if let Some(MicropubPropertyValue::Value(url)) = props.remove("value") {
                        let alt = match props.remove("alt") {
                            Some(MicropubPropertyValue::Value(alt)) => Some(alt),
                            _ => None,
                        };
                        let photo = Photo { url, alt };
                        self.add_photo(photo);
                    }
                }
            }
            MicropubPropertyValue::ValueVec(photos) => {
                for photo in photos {
                    self.handle_photo(photo);
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MicropubForm {
    /// Access token (token used to authenticate the operation).
    /// May be used in place of a bearer token authorization header.
    pub access_token: Option<String>,

    /// Entry type
    pub h: String,

    /// Text content of the entry
    pub content: String,

    /// Content type of the entry. None for plain text / default, "html" for already rendered html,
    /// or "markdown" for content that should be rendered as html from markdown at post render
    /// time.
    pub content_type: Option<String>,

    /// Categories (tags) for the entry
    pub category: Vec<String>,

    /// Name/Title of the h-entry (article/blog post).
    /// Note that h-notes do not contain a name.
    pub name: Option<String>,

    /// Created and Updated at datetimes of the post
    /// The database schema has a default of the current time but this can also be provided at post
    /// time.
    pub created_at: Option<String>,
    pub updated_at: Option<String>,

    /// Slug to use as part of URI
    pub slug: Option<String>,

    /// Indicates entry is a bookmark type. String should be a URL.
    pub bookmark_of: Option<String>,

    /// Photos included with the entry
    pub photos: Option<Vec<Photo>>,
    // TODO: support additional fields and properties
}

impl MicropubForm {
    pub fn from_form_bytes(b: &[u8]) -> Result<Self> {
        let p = parse(b);
        let mut builder = MicropubFormBuilder::new();
        for (k, v) in p {
            match &*k {
                "access_token" => builder.set_access_token(v.into_owned()),
                "h" => builder.set_h(v.into_owned()),
                content_key @ ("content" | "content[html]") => {
                    builder.set_content(v.into_owned());
                    if content_key == "content[html]" {
                        builder.set_content_type("html".into());
                    }
                }
                "category" | "category[]" => builder.add_category(v.into_owned()),
                "name" => builder.set_name(v.into_owned()),
                "bookmark-of" => builder.set_bookmark_of(v.into_owned()),
                _ => (),
            }
        }

        Ok(builder.build()?)
    }

    pub fn from_json_bytes(b: &[u8]) -> Result<Self> {
        Ok(MicropubFormBuilder::from_json(b)?.build()?)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn micropub_form_decode_category_as_array() {
        // arrange
        let qs = b"h=entry&content=this+is+only+a+test+of+micropub&category%5B%5D=test&category%5B%5D=micropub";

        // act
        let form = MicropubForm {
            access_token: None,
            name: None,
            h: "entry".into(),
            content: "this is only a test of micropub".into(),
            content_type: None,
            category: vec!["test".into(), "micropub".into()],
            created_at: None,
            updated_at: None,
            slug: None,
            bookmark_of: None,
            photos: None,
        };

        // assert
        assert_eq!(form, MicropubForm::from_form_bytes(&qs[..]).unwrap());
    }

    #[test]
    fn micropub_form_decode_category_as_single_param_into_vec() {
        // arrange
        let qs = b"h=entry&content=this+is+only+a+test+of+micropub&category=micropub";

        // act
        let form = MicropubForm {
            access_token: None,
            name: None,
            h: "entry".into(),
            content: "this is only a test of micropub".into(),
            content_type: None,
            category: vec!["micropub".into()],
            created_at: None,
            updated_at: None,
            slug: None,
            bookmark_of: None,
            photos: None,
        };

        // assert
        assert_eq!(form, MicropubForm::from_form_bytes(&qs[..]).unwrap());
    }

    #[test]
    fn micropub_form_decode_category_missing_empty_vec() {
        // arrange
        let qs = b"h=entry&content=this+is+only+a+test+of+micropub";

        // act
        let form = MicropubForm {
            access_token: None,
            name: None,
            h: "entry".into(),
            content: "this is only a test of micropub".into(),
            content_type: None,
            category: vec![],
            created_at: None,
            updated_at: None,
            slug: None,
            bookmark_of: None,
            photos: None,
        };

        // assert
        assert_eq!(form, MicropubForm::from_form_bytes(&qs[..]).unwrap());
    }

    #[test]
    fn micropub_form_decode_content_html() {
        // arrange
        let qs = b"h=entry&name=Test%20Article%20from%20Micropublish.net&content[html]=%3Cdiv%3EThis%20is%20a%20test%20article%3Cbr%3E%3Cbr%3E%3Cstrong%3EIt%20has%20formatting%3Cbr%3E%3Cbr%3E%3C%2Fstrong%3EIt%20can%20%3Ca%20href%3D%22https%3A%2F%2Fwww.egoroff.spb.ru%22%3Eembed%20links%3C%2Fa%3E%3C%2Fdiv%3E&category=test&post-status=published&mp-slug=test-article-micropublish-net";

        // act
        let form = MicropubForm {
            access_token: None,
            name: Some("Test Article from Micropublish.net".into()),
            h: "entry".into(),
            content: "<div>This is a test article<br><br><strong>It has formatting<br><br></strong>It can <a href=\"https://www.egoroff.spb.ru\">embed links</a></div>".into(),
            content_type: Some("html".into()),
            category: vec!["test".into()],
            created_at: None,
            updated_at: None,
            slug: None,
            bookmark_of: None,
            photos: None,
        };

        // assert
        assert_eq!(form, MicropubForm::from_form_bytes(&qs[..]).unwrap());
    }

    #[test]
    fn micropub_json_decode_post_entry_from_quill() {
        // arrange
        let bytes = b"{\"type\":[\"h-entry\"],\"properties\":{\"name\":[\"Testing quill\"],\"content\":[{\"html\":\"<p>This is a test of https:\\/\\/quill.p3k.io<\\/p>\\n<p>\\n  hello hello\\n  <br \\/>\\n<\\/p>\"}],\"category\":[\"test\"],\"mp-slug\":[\"quill-test\"]}}";

        // act
        let form = MicropubForm {
            access_token: None,
            name: Some("Testing quill".into()),
            h: "entry".into(),
            content:
                "<p>This is a test of https://quill.p3k.io</p>\n<p>\n  hello hello\n  <br />\n</p>"
                    .into(),
            content_type: Some("html".into()),
            category: vec!["test".into()],
            created_at: None,
            updated_at: None,
            slug: Some("quill-test".into()),
            bookmark_of: None,
            photos: None,
        };

        // assert
        assert_eq!(form, MicropubForm::from_json_bytes(&bytes[..]).unwrap());
    }

    #[test]
    fn micropub_json_decode_bookmark_of_entry() {
        // arrange
        let bytes = b"{\"type\":[\"h-entry\"],\"properties\":{\"name\":[\"Testing bookmarks\"],\"content\":[\"Bookmark test\"],\"bookmark-of\":[\"https://www.egoroff.spb.ru\"]}}";

        // act
        let form = MicropubForm {
            access_token: None,
            name: Some("Testing bookmarks".into()),
            h: "entry".into(),
            content: "Bookmark test".into(),
            content_type: None,
            category: vec![],
            created_at: None,
            updated_at: None,
            slug: None,
            bookmark_of: Some("https://www.egoroff.spb.ru".into()),
            photos: None,
        };

        // assert
        assert_eq!(form, MicropubForm::from_json_bytes(&bytes[..]).unwrap());
    }

    #[test]
    fn micropub_json_decode_post_entry_markdown_format() {
        // arrange
        let bytes = b"{\"type\":[\"h-entry\"],\"properties\":{\"name\":[\"Testing markdown\"],\"content\":[{\"markdown\":\"This _is_ a *markdown* document. \\n # Header 1 \\n normal text\"}],\"category\":[\"markdown\"],\"mp-slug\":[\"markdown-test\"]}}";

        // act
        let form = MicropubForm {
            access_token: None,
            name: Some("Testing markdown".into()),
            h: "entry".into(),
            content: "This _is_ a *markdown* document. \n # Header 1 \n normal text".into(),
            content_type: Some("markdown".into()),
            category: vec!["markdown".into()],
            created_at: None,
            updated_at: None,
            slug: Some("markdown-test".into()),
            bookmark_of: None,
            photos: None,
        };

        // assert
        assert_eq!(form, MicropubForm::from_json_bytes(&bytes[..]).unwrap());
    }

    #[test]
    fn micropub_json_decode_handles_published_property() {
        // arrange
        let bytes = b"{\"type\":[\"h-entry\"],\"properties\":{\"name\":[\"Testing published\"],\"content\":[{\"html\":\"content!\"}],\"category\":[\"publish-date\"],\"mp-slug\":[\"publish-date-slug\"], \"published\":[\"2020-04-04 15:30:00\"]}}";

        // act
        let form = MicropubForm {
            access_token: None,
            name: Some("Testing published".into()),
            h: "entry".into(),
            content: "content!".into(),
            content_type: Some("html".into()),
            category: vec!["publish-date".into()],
            created_at: Some("2020-04-04 15:30:00".into()),
            updated_at: None,
            slug: Some("publish-date-slug".into()),
            bookmark_of: None,
            photos: None,
        };

        // assert
        assert_eq!(form, MicropubForm::from_json_bytes(&bytes[..]).unwrap());
    }

    #[test]
    fn micropub_form_decode_photo_property() {
        // arrange
        let bytes = b"{\"type\":[\"h-entry\"],\"properties\":{\"content\":[\"test upload\"],\"photo\":[{\"value\":\"https:\\/\\/www.egoroff.spb.ru\\/media\\/2a2ae02f9addf60f708298221e661db15b8afc340d8b934bc94b9e37f293074f\",\"alt\":\"test upload\"}]}}";

        // act
        let form = MicropubForm {
            access_token: None,
            name: None,
            h: "entry".into(),
            content: "test upload".into(),
            content_type: None,
            category: vec![],
            created_at: None,
            updated_at: None,
            slug: None,
            bookmark_of: None,
            photos: Some(vec![
                Photo {
                    url: "https://www.egoroff.spb.ru/media/2a2ae02f9addf60f708298221e661db15b8afc340d8b934bc94b9e37f293074f".into(),
                    alt: Some("test upload".into()),
                }
            ]),
        };

        // assert
        assert_eq!(form, MicropubForm::from_json_bytes(&bytes[..]).unwrap());
    }

    #[test]
    fn micropub_form_decode_multiple_photo_property() {
        // arrange
        let bytes = b"{\"type\":[\"h-entry\"],\"properties\":{\"content\":[\"test upload\"],\"photo\":[{\"value\":\"https:\\/\\/www.egoroff.spb.ru\\/media\\/2a2ae02f9addf60f708298221e661db15b8afc340d8b934bc94b9e37f293074f\",\"alt\":\"test upload\"},\"https:\\/\\/www.egoroff.spb.ru\\/media\\/df1dfea9b0a062e8e27ee6fed1df597995547e16a73570107ff475b33d59f4fb\"]}}";

        // act
        let form = MicropubForm {
            access_token: None,
            name: None,
            h: "entry".into(),
            content: "test upload".into(),
            content_type: None,
            category: vec![],
            created_at: None,
            updated_at: None,
            slug: None,
            bookmark_of: None,
            photos: Some(vec![
                Photo {
                    url: "https://www.egoroff.spb.ru/media/2a2ae02f9addf60f708298221e661db15b8afc340d8b934bc94b9e37f293074f".into(),
                    alt: Some("test upload".into()),
                },
                Photo {
                    url: "https://www.egoroff.spb.ru/media/df1dfea9b0a062e8e27ee6fed1df597995547e16a73570107ff475b33d59f4fb".into(),
                    alt: None,
                }
            ]),
        };

        // assert
        assert_eq!(form, MicropubForm::from_json_bytes(&bytes[..]).unwrap());
    }
}
