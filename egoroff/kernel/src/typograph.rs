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
                text!("dd", handler),
            ],
            document_content_handlers: vec![],
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
    #[case("<pre>a - b</pre>", "<pre>a - b</pre>")]
    fn typograph_tests(#[case] str: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = typograph(str.to_string());

        // assert
        assert_eq!(expected, actual);
    }
}
