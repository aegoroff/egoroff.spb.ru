use anyhow::Result;
use lol_html::html_content::Element;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::{borrow::Cow, iter};

use lol_html::{
    ElementContentHandlers, HtmlRewriter, Selector, Settings, element,
    html_content::{ContentType, TextChunk, TextType},
    text,
};
use regex::Regex;

const ALLOWED_TAGS: &[&str] = &[
    "p", "div", "span", "a", "dt", "dd", "li", "i", "b", "em", "strong", "small", "h1", "h2", "h3",
    "h4", "h5", "h6", "td", "th",
];

// Combine similar patterns for optimization
static TYPOGRAPH_RE: std::sync::LazyLock<Result<Vec<Regex>, regex::Error>> =
    std::sync::LazyLock::new(|| {
        Ok(vec![
            // Replace spaces before hyphens
            Regex::new(r"(\w)-(\s+)")?,
            // Replace plus-minus
            Regex::new(r"\+-")?,
            // Replace dashes with spaces
            Regex::new(r"(\s+)(--?|—|-)(\s|\u00a0)")?,
            // Replace dash at the beginning of a line
            Regex::new(r"(^)(--?|—|-)(\s|\u00a0)")?,
            // Replace ellipsis
            Regex::new(r"\.{2,}")?,
            // Replace minus between digits
            Regex::new(r"(\d)-(\d)")?,
            // Replace opening quotes
            Regex::new(r#"["»](\S)"#)?,
            // Replace closing quotes
            Regex::new(r#"(\S)["«]"#)?,
        ])
    });

static ALLOWED_SET: std::sync::LazyLock<HashSet<&'static str>> =
    std::sync::LazyLock::new(|| ALLOWED_TAGS.iter().copied().collect());

pub fn typograph(html: &str) -> Result<String> {
    let stack = Rc::new(RefCell::new(Vec::<String>::with_capacity(32)));

    let text_handler = |t: &mut TextChunk| {
        if t.text_type() != TextType::Data {
            return Ok(());
        }

        if let Some(tag) = stack.borrow().last() {
            if !ALLOWED_SET.contains(tag.as_str()) {
                return Ok(());
            }
        }

        let mut text = t.as_str().to_string();

        // Apply all replacements in one pass
        if let Ok(regexes) = TYPOGRAPH_RE.as_ref() {
            for (i, re) in regexes.iter().enumerate() {
                text = match i {
                    0 => re.replace_all(&text, "$1 -$2").into_owned(),
                    1 => re.replace_all(&text, "&plusmn;").into_owned(),
                    2 => re.replace_all(&text, "&nbsp;&mdash;$3").into_owned(),
                    3 => re.replace_all(&text, "&mdash;$3").into_owned(),
                    4 => re.replace_all(&text, "&hellip;").into_owned(),
                    5 => re.replace_all(&text, "$1&minus;$2").into_owned(),
                    6 => re.replace_all(&text, "«$1").into_owned(),
                    7 => re.replace_all(&text, "$1»").into_owned(),
                    _ => text,
                };
            }
        }

        t.replace(&text, ContentType::Html);
        Ok(())
    };

    let element_handler: (Cow<Selector>, ElementContentHandlers) =
        element!("*", |e: &mut Element| {
            let tag_name = e.tag_name();
            stack.borrow_mut().push(tag_name);

            if let Some(handlers) = e.end_tag_handlers() {
                let stack = stack.clone();
                handlers.push(Box::new(move |_end| {
                    stack.borrow_mut().pop();
                    Ok(())
                }));
            } else {
                stack.borrow_mut().pop();
            }

            Ok(())
        });

    let handlers: Vec<(Cow<Selector>, ElementContentHandlers)> = ALLOWED_TAGS
        .iter()
        .map(|t| text!(*t, text_handler))
        .chain(iter::once(element_handler))
        .collect();

    let mut result = Vec::with_capacity(html.len() * 2); // Pre-allocate more memory

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: handlers,
            ..Settings::default()
        },
        |c: &[u8]| result.extend(c),
    );

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    Ok(String::from_utf8(result)?)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("<p>a - b</p>", "<p>a&nbsp;&mdash; b</p>")]
    #[case("<p>a - b</p><br/>", "<p>a&nbsp;&mdash; b</p><br/>")]
    #[case(
        "<div>a - b<code>c - d</code>e - f</div>",
        "<div>a&nbsp;&mdash; b<code>c - d</code>e&nbsp;&mdash; f</div>"
    )]
    #[case(
        "<div>a - b<br/><code>c - d</code>e - f</div>",
        "<div>a&nbsp;&mdash; b<br/><code>c - d</code>e&nbsp;&mdash; f</div>"
    )]
    #[case(
        "<div>a - b<code><![CDATA[c - d]]></code>e - f</div>",
        "<div>a&nbsp;&mdash; b<code><![CDATA[c - d]]></code>e&nbsp;&mdash; f</div>"
    )]
    #[case(
        "<div>a - b<script>c - d</script>e - f</div>",
        "<div>a&nbsp;&mdash; b<script>c - d</script>e&nbsp;&mdash; f</div>"
    )]
    #[case("<p>a - b c - d</p>", "<p>a&nbsp;&mdash; b c&nbsp;&mdash; d</p>")]
    #[case("<p>- b</p>", "<p>&mdash; b</p>")]
    #[case("<p>- b..</p>", "<p>&mdash; b&hellip;</p>")]
    #[case("<p>- b 1-2</p>", "<p>&mdash; b 1&minus;2</p>")]
    #[case("<p>a- b</p>", "<p>a&nbsp;&mdash; b</p>")]
    #[case("<p>a- b+-</p>", "<p>a&nbsp;&mdash; b&plusmn;</p>")]
    #[case("<div>a - b</div>", "<div>a&nbsp;&mdash; b</div>")]
    #[case("<span>a - b</span>", "<span>a&nbsp;&mdash; b</span>")]
    #[case("<a>a - b</a>", "<a>a&nbsp;&mdash; b</a>")]
    #[case("<dd>a - b</dd>", "<dd>a&nbsp;&mdash; b</dd>")]
    #[case("<dt>a - b</dt>", "<dt>a&nbsp;&mdash; b</dt>")]
    #[case("<li>a - b</li>", "<li>a&nbsp;&mdash; b</li>")]
    #[case("<i>a - b</i>", "<i>a&nbsp;&mdash; b</i>")]
    #[case("<b>a - b</b>", "<b>a&nbsp;&mdash; b</b>")]
    #[case("<td>a - b</td>", "<td>a&nbsp;&mdash; b</td>")]
    #[case("<th>a - b</th>", "<th>a&nbsp;&mdash; b</th>")]
    #[case("<em>a - b</em>", "<em>a&nbsp;&mdash; b</em>")]
    #[case("<h1>a - b</h1>", "<h1>a&nbsp;&mdash; b</h1>")]
    #[case("<h2>a - b</h2>", "<h2>a&nbsp;&mdash; b</h2>")]
    #[case("<h3>a - b</h3>", "<h3>a&nbsp;&mdash; b</h3>")]
    #[case("<h4>a - b</h4>", "<h4>a&nbsp;&mdash; b</h4>")]
    #[case("<h5>a - b</h5>", "<h5>a&nbsp;&mdash; b</h5>")]
    #[case("<h6>a - b</h6>", "<h6>a&nbsp;&mdash; b</h6>")]
    #[case("<small>a - b</small>", "<small>a&nbsp;&mdash; b</small>")]
    #[case("<strong>a - b</strong>", "<strong>a&nbsp;&mdash; b</strong>")]
    #[case("<pre>a - b</pre>", "<pre>a - b</pre>")]
    #[case(
        "<i>a - b</i> <b>c - d</b>",
        "<i>a&nbsp;&mdash; b</i> <b>c&nbsp;&mdash; d</b>"
    )]
    #[case(
        "<i>a - b</i> <b>c -- d</b>",
        "<i>a&nbsp;&mdash; b</i> <b>c&nbsp;&mdash; d</b>"
    )]
    #[case("<p>test \"a\"bc</p>", "<p>test «a»bc</p>")]
    #[case("<p>URL \"/\". </p>", "<p>URL «/». </p>")]
    #[case("<p>test \"a - b\"cd</p>", "<p>test «a&nbsp;&mdash; b»cd</p>")]
    fn typograph_tests(#[case] str: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = typograph(str).unwrap();

        // assert
        assert_eq!(expected, actual);
    }
}
