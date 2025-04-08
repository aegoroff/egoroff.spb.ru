use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    io::Cursor,
    str,
};

use anyhow::Result;
use lol_html::{ElementContentHandlers, HtmlRewriter, Selector, Settings, element, text};
use pulldown_cmark::{Options, Parser, html};
use quick_xml::{
    Reader, Writer,
    events::{BytesEnd, BytesStart, Event},
};

const REPLACES: &[(&[u8], &str)] = &[
    (b"example", "pre"),
    (b"quote", "blockquote"),
    (b"link", "a"),
    (b"center", "div"),
    (b"div1", "2"),
    (b"div2", "3"),
    (b"div3", "4"),
    (b"head", "h"),
    (b"table", "table"),
    (b"acronym", "acronym"),
];

const PARENTS: &[&[u8]] = &[b"div1", b"div2", b"div3"];

static PARENTS_SET: std::sync::LazyLock<HashSet<&'static [u8]>> =
    std::sync::LazyLock::new(|| PARENTS.iter().copied().collect());

static REPLACES_MAP: std::sync::LazyLock<HashMap<&'static [u8], &'static str>> =
    std::sync::LazyLock::new(|| REPLACES.iter().map(|(k, v)| (*k, *v)).collect());

pub fn xml2html(input: &str) -> Result<String> {
    let mut reader = Reader::from_str(input);
    let mut writer = Writer::new(Cursor::new(Vec::with_capacity(input.len())));

    let mut parent = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if REPLACES_MAP.contains_key(e.name().as_ref()) => {
                let replace = REPLACES_MAP.get(e.name().as_ref()).unwrap_or(&"");
                if PARENTS_SET.contains(e.name().as_ref()) {
                    parent = String::from(*replace);
                    continue;
                }

                let mut elem = if *replace == "h" {
                    BytesStart::new(format!("h{parent}"))
                } else {
                    BytesStart::new(*replace)
                };

                let original_attributes = e.attributes().filter_map(std::result::Result::ok);
                if *replace == "a" {
                    let mut href = String::new();
                    original_attributes.for_each(|a| {
                        let attr = a.key.local_name();
                        let id = str::from_utf8(attr.as_ref()).unwrap_or("");
                        let val = str::from_utf8(a.value.as_ref()).unwrap_or("");
                        href = match id {
                            "id" => match val {
                                "1" | "53" | "62" => "/portfolio/".to_string(),
                                "2" => "/blog/".to_string(),
                                _ => "/".to_string(),
                            },
                            "hame" => format!("/blog/{val}.html"),
                            _ => String::new(),
                        };
                    });
                    if !href.is_empty() {
                        elem.push_attribute(("href", href.as_str()));
                    }
                    elem.push_attribute(("itemprop", "url"));
                } else {
                    elem.extend_attributes(original_attributes);
                }

                if *replace == "table" {
                    elem.push_attribute(("class", "table table-condensed table-striped"));
                }
                if *replace == "acronym" {
                    elem.push_attribute(("class", "initialism"));
                }

                writer.write_event(Event::Start(elem))?;
            }
            Ok(Event::End(e)) if REPLACES_MAP.contains_key(e.name().as_ref()) => {
                if PARENTS_SET.contains(e.name().as_ref()) {
                    continue;
                }

                let replace = REPLACES_MAP.get(e.name().as_ref()).unwrap_or(&"");
                let elem = if *replace == "h" {
                    BytesEnd::new(format!("h{parent}"))
                } else {
                    BytesEnd::new(*replace)
                };

                writer.write_event(Event::End(elem))?;
            }
            Ok(Event::CData(e)) => {
                let escaped = e.escape()?;
                let evt = Event::Text(escaped);
                writer.write_event(evt)?;
            }
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(e) => writer.write_event(e)?,
            Err(e) => return Err(anyhow::Error::new(e)),
        }
    }

    let result = writer.into_inner().into_inner();
    let result = String::from_utf8(result)?;
    Ok(result)
}

pub fn markdown2html(input: &str) -> Result<String> {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(input, options);
    let mut html = String::with_capacity(input.len() * 2);
    html::push_html(&mut html, parser);

    let table_handler: (Cow<Selector>, ElementContentHandlers) = element!("table", |e| {
        e.set_attribute("class", "table table-condensed table-striped")
            .unwrap_or_default();
        Ok(())
    });

    let mut result = Vec::with_capacity(html.len());
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![table_handler],
            ..Settings::default()
        },
        |c: &[u8]| {
            result.extend(c);
        },
    );

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;
    let result = String::from_utf8(result)?;
    Ok(result)
}

pub fn html2text(html: &str) -> Result<String> {
    let mut text = Vec::with_capacity(64);

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![text!("*", |t| {
                if !t.as_str().is_empty() {
                    text.push(t.as_str().to_string());
                }

                Ok(())
            })],
            ..Settings::default()
        },
        |_: &[u8]| {},
    );

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    Ok(text.join(" "))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("<p>test \"a - b\"cd</p>", "<p>test \"a - b\"cd</p>")]
    #[case(
        "<?xml version=\"1.0\"?><p>test \"a - b\"cd</p>",
        "<?xml version=\"1.0\"?><p>test \"a - b\"cd</p>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><center>test</center>",
        "<?xml version=\"1.0\"?><div>test</div>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><example>test</example>",
        "<?xml version=\"1.0\"?><pre>test</pre>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><example><![CDATA[<p>test</p>]]></example>",
        "<?xml version=\"1.0\"?><pre>&lt;p&gt;test&lt;/p&gt;</pre>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><example class=\"lang-rust\">test</example>",
        "<?xml version=\"1.0\"?><pre class=\"lang-rust\">test</pre>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><quote>test</quote>",
        "<?xml version=\"1.0\"?><blockquote>test</blockquote>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><link>test</link>",
        "<?xml version=\"1.0\"?><a itemprop=\"url\">test</a>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><div1><head>test</head><p>b</p></div1>",
        "<?xml version=\"1.0\"?><h2>test</h2><p>b</p>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><div2><head>test</head><p>b</p></div2>",
        "<?xml version=\"1.0\"?><h3>test</h3><p>b</p>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><div3><head>test</head><p>b</p></div3>",
        "<?xml version=\"1.0\"?><h4>test</h4><p>b</p>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><div1><head>h</head><div2><head>h</head></div2></div1>",
        "<?xml version=\"1.0\"?><h2>h</h2><h3>h</h3>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><table><tr><td>test</td></tr></table>",
        "<?xml version=\"1.0\"?><table class=\"table table-condensed table-striped\"><tr><td>test</td></tr></table>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><acronym>test</acronym>",
        "<?xml version=\"1.0\"?><acronym class=\"initialism\">test</acronym>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><link id=\"3\">test</link>",
        "<?xml version=\"1.0\"?><a href=\"/\" itemprop=\"url\">test</a>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><link id=\"2\">test</link>",
        "<?xml version=\"1.0\"?><a href=\"/blog/\" itemprop=\"url\">test</a>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><link hame=\"2\">test</link>",
        "<?xml version=\"1.0\"?><a href=\"/blog/2.html\" itemprop=\"url\">test</a>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><link id=\"1\">test</link>",
        "<?xml version=\"1.0\"?><a href=\"/portfolio/\" itemprop=\"url\">test</a>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><link id=\"53\">test</link>",
        "<?xml version=\"1.0\"?><a href=\"/portfolio/\" itemprop=\"url\">test</a>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><link id=\"62\">test</link>",
        "<?xml version=\"1.0\"?><a href=\"/portfolio/\" itemprop=\"url\">test</a>"
    )]
    #[case(
        "<link id=\"62\">test</link>",
        "<a href=\"/portfolio/\" itemprop=\"url\">test</a>"
    )]
    #[case(
        "<link id=\"62\">test",
        "<a href=\"/portfolio/\" itemprop=\"url\">test"
    )]
    #[case("a", "a")]
    fn xml2html_tests(#[case] test_data: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = xml2html(test_data).unwrap();

        // assert
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("# a\nb", "<h1>a</h1>\n<p>b</p>\n")]
    #[case("## a\nb", "<h2>a</h2>\n<p>b</p>\n")]
    #[case("1. a\n2. b", "<ol>\n<li>a</li>\n<li>b</li>\n</ol>\n")]
    #[case("- a\n- b", "<ul>\n<li>a</li>\n<li>b</li>\n</ul>\n")]
    fn markdown2html_tests(#[case] test_data: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = markdown2html(test_data).unwrap();

        // assert
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("<h1>a</h1>\n<p>b</p>\n", "a b")]
    #[case("a<h1>b</h1>\n<p>c</p>d\n", "b c")]
    fn html2text_tests(#[case] test_data: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = html2text(test_data).unwrap();

        // assert
        assert_eq!(expected, actual);
    }
}
