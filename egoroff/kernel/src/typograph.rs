use std::collections::HashSet;
use std::{borrow::Cow, iter};

use lol_html::{
    element,
    html_content::{ContentType, TextChunk, TextType},
    text, ElementContentHandlers, HtmlRewriter, Selector, Settings,
};
use regex::Regex;

const ALLOWED_TAGS: &[&str] = &[
    "p", "div", "span", "a", "dt", "dd", "li", "i", "b", "em", "strong", "small", "h1", "h2", "h3",
    "h4", "h5", "h6", "td", "th",
];

pub fn typograph(str: String) -> String {
    let mut result = vec![];

    let output_sink = |c: &[u8]| {
        result.extend(c);
    };

    let spaces = Regex::new(r"(\w)-(\s+)").unwrap();
    let plusmn = Regex::new(r"\+-").unwrap();
    let nbsp = Regex::new(r"(\s+)(--?|—|-)(\s|\u00a0)").unwrap();
    let mdash = Regex::new(r"(^)(--?|—|-)(\s|\u00a0)").unwrap();
    let hellip = Regex::new(r"\.{2,}").unwrap();
    let minus_beetween_digits = Regex::new(r"(\d)-(\d)").unwrap();
    let open_quote = Regex::new(r#"["»](\S)"#).unwrap();
    let close_quote = Regex::new(r#"(\S)["«]"#).unwrap();

    let stack = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));

    let allowed: HashSet<&&str> = ALLOWED_TAGS.iter().collect();

    let text_handler = |t: &mut TextChunk| {
        if t.text_type() != TextType::Data {
            return Ok(());
        }

        if let Some(t) = stack.borrow().last() {
            if !allowed.contains(&t.as_str()) {
                return Ok(());
            }
        }

        let original = t.as_str().to_string();

        let replace = spaces.replace_all(&original, "$1 -$2");
        let replace = plusmn.replace_all(&replace, "&plusmn;");
        let replace = nbsp.replace_all(&replace, "&nbsp;&mdash;$3");
        let replace = mdash.replace_all(&replace, "&mdash;$3");
        let replace = hellip.replace_all(&replace, "&hellip;");
        let replace = minus_beetween_digits.replace_all(&replace, "$1&minus;$2");
        let replace = open_quote.replace_all(&replace, "«$1");
        let replace = close_quote.replace_all(&replace, "$1»");
        t.replace(&replace, ContentType::Html);

        Ok(())
    };

    let element_handler: (Cow<Selector>, ElementContentHandlers) = element!("*", |e| {
        stack.borrow_mut().push(e.tag_name());
        let to_use_if_err = stack.clone();
        let stack = stack.clone();
        let end_tag_handler = e.on_end_tag(move |_end| {
            stack.borrow_mut().pop();
            Ok(())
        });
        if end_tag_handler.is_err() {
            to_use_if_err.borrow_mut().pop();
        }

        Ok(())
    });

    let handlers: Vec<(Cow<Selector>, ElementContentHandlers)> = ALLOWED_TAGS
        .iter()
        .map(|t| text!(*t, text_handler))
        .chain(iter::once(element_handler))
        .collect();

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: handlers,
            ..Settings::default()
        },
        output_sink,
    );

    rewriter.write(str.as_bytes()).unwrap();
    rewriter.end().unwrap();

    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

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
        let actual = typograph(str.to_string());

        // assert
        assert_eq!(expected, actual);
    }
}
