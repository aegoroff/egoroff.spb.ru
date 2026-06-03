use anyhow::Result;
use lol_html::html_content::Element;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

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

type Rule = (&'static str, &'static str);

static RULES: &[Rule] = &[
    (r"(\w)-(\s+)", "$1 -$2"),
    (r"\+-", "&plusmn;"),
    (r"(\s+)(--?|—|-)(\s|\u00a0)", "&nbsp;&mdash;$3"),
    (r"(^)(--?|—|-)(\s|\u00a0)", "&mdash;$3"),
    (r"\.{2,}", "&hellip;"),
    (r"(\d)-(\d)", "$1&minus;$2"),
    (r#"["»](\S)"#, "«$1"),
    (r#"(\S)["«]"#, "$1»"),
];

static TYPOGRAPH_RE: std::sync::LazyLock<Vec<(Regex, &'static str)>> =
    std::sync::LazyLock::new(|| {
        RULES
            .iter()
            .map(|(pat, repl)| (Regex::new(pat).expect("invalid regex"), *repl))
            .collect()
    });

static ALLOWED_SET: std::sync::LazyLock<HashSet<&'static str>> =
    std::sync::LazyLock::new(|| ALLOWED_TAGS.iter().copied().collect());

pub fn typograph(html: &str) -> Result<String> {
    let forbidden_depth: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

    let text_handler = |t: &mut TextChunk| {
        if t.text_type() != TextType::Data {
            return Ok(());
        }

        if *forbidden_depth.borrow() > 0 {
            return Ok(());
        }

        let mut text = t.as_str().to_string();

        // Apply all replacements in one pass
        for (re, replacement) in TYPOGRAPH_RE.iter() {
            text = re.replace_all(&text, *replacement).into_owned();
        }

        if text != t.as_str() {
            t.replace(&text, ContentType::Html);
        }
        Ok(())
    };

    let element_handler: (Cow<Selector>, ElementContentHandlers) =
        element!("*", |e: &mut Element| {
            let tag_name = e.tag_name();
            let is_forbidden = !ALLOWED_SET.contains(tag_name.as_str());

            if is_forbidden && let Some(handlers) = e.end_tag_handlers() {
                *forbidden_depth.borrow_mut() += 1;
                let depth = forbidden_depth.clone();
                handlers.push(Box::new(move |_| {
                    *depth.borrow_mut() -= 1;
                    Ok(())
                }));
                // self-closing (<br/>, <img/> и т.д.) — ignore
            }

            Ok(())
        });

    let mut settings = Settings::new();
    for t in ALLOWED_TAGS {
        settings = settings.append_element_content_handler(text!(*t, text_handler));
    }

    let mut result = Vec::with_capacity(html.len() * 2); // Pre-allocate more memory

    let mut rewriter = HtmlRewriter::new(
        settings.append_element_content_handler(element_handler),
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
