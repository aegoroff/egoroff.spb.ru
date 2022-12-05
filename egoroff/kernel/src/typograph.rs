use lol_html::{
    html_content::{ContentType, TextChunk},
    text, HtmlRewriter, Settings,
};
use regex::Regex;

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

    let handler = |t: &mut TextChunk| {
        let original = t.as_str().to_string();
        let replace = spaces.replace_all(&original, "$1 -$2");
        let replace = plusmn.replace_all(&replace, "&plusmn;");
        let replace = nbsp.replace_all(&replace, "&nbsp;&mdash;$3");
        let replace = mdash.replace_all(&replace, "&mdash;$3");
        let replace = hellip.replace_all(&replace, "&hellip;");
        let replace = minus_beetween_digits.replace_all(&replace, "$1&minus;$2");
        t.replace(&replace, ContentType::Html);

        Ok(())
    };

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                text!("p", handler),
                text!("div", handler),
                text!("span", handler),
                text!("a", handler),
                text!("dt", handler),
                text!("dd", handler),
                text!("li", handler),
                text!("i", handler),
                text!("b", handler),
                text!("em", handler),
                text!("strong", handler),
                text!("h1", handler),
                text!("h2", handler),
                text!("h3", handler),
                text!("h4", handler),
                text!("h5", handler),
                text!("h6", handler),
                text!("td", handler),
                text!("th", handler),
                text!("small", handler),
            ],
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
    #[case("<i>a - b</i> <b>c - d</b>", "<i>a&nbsp;&mdash; b</i> <b>c&nbsp;&mdash; d</b>")]
    #[case("<i>a - b</i> <b>c -- d</b>", "<i>a&nbsp;&mdash; b</i> <b>c&nbsp;&mdash; d</b>")]
    fn typograph_tests(#[case] str: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = typograph(str.to_string());

        // assert
        assert_eq!(expected, actual);
    }
}
