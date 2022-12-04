use lol_html::{
    html_content::{ContentType, TextChunk},
    text, HtmlRewriter, Settings,
};
use regex::Regex;

pub fn typograph(str: String) -> String {
    let mut result = vec![];

    // Use stdout as an output sink for the rewriter
    let output_sink = |c: &[u8]| {
        result.extend(c);
    };

    let nbsp = Regex::new(r"(\s+)(--?|—|-)(\s|\u00a0)").unwrap();

    let handler = |t: &mut TextChunk| {
        let original = t.as_str().to_string();
        let replace = nbsp.replace_all(&original, "&nbsp;&mdash;$3");
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
