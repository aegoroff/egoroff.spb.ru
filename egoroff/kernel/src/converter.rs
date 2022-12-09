use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
};

use quick_xml::{
    events::{BytesEnd, BytesStart, Event},
    Reader, Writer,
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

pub fn xml2html(input: String) -> String {
    if !input.starts_with("<?xml version=\"1.0\"?>") {
        return input;
    }
    let mut reader = Reader::from_str(&input);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let parents: HashSet<&[u8]> = PARENTS.iter().map(|i| *i).collect();
    let replaces: HashMap<&[u8], &str> = REPLACES.iter().map(|(k, v)| (*k, *v)).collect();

    let mut parent = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if replaces.contains_key(e.name().as_ref()) => {
                let replace = replaces.get(e.name().as_ref()).unwrap();
                if parents.contains(e.name().as_ref()) {
                    parent = String::from(*replace);
                    continue;
                }

                let mut elem = if *replace == "h" {
                    let new_tag = String::from("h");
                    BytesStart::new(new_tag + &parent)
                } else {
                    BytesStart::new(*replace)
                };

                elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));

                if *replace == "table" {
                    elem.push_attribute(("class", "table table-condensed table-striped"));
                }
                if *replace == "acronym" {
                    elem.push_attribute(("class", "initialism"));
                }

                assert!(writer.write_event(Event::Start(elem)).is_ok());
            }
            Ok(Event::End(e)) if replaces.contains_key(e.name().as_ref()) => {
                if parents.contains(e.name().as_ref()) {
                    continue;
                }

                let replace = replaces.get(e.name().as_ref()).unwrap();
                let elem = if *replace == "h" {
                    let new_tag = String::from("h");
                    BytesEnd::new(new_tag + &parent)
                } else {
                    BytesEnd::new(*replace)
                };

                assert!(writer.write_event(Event::End(elem)).is_ok());
            }
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(e) => assert!(writer.write_event(e).is_ok()),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

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
        "<?xml version=\"1.0\"?><example class=\"lang-rust\">test</example>",
        "<?xml version=\"1.0\"?><pre class=\"lang-rust\">test</pre>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><quote>test</quote>",
        "<?xml version=\"1.0\"?><blockquote>test</blockquote>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><link>test</link>",
        "<?xml version=\"1.0\"?><a>test</a>"
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
        "<?xml version=\"1.0\"?><table><tr><td>test</td></tr></table>",
        "<?xml version=\"1.0\"?><table class=\"table table-condensed table-striped\"><tr><td>test</td></tr></table>"
    )]
    #[case(
        "<?xml version=\"1.0\"?><acronym>test</acronym>",
        "<?xml version=\"1.0\"?><acronym class=\"initialism\">test</acronym>"
    )]
    fn converter_tests(#[case] str: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = xml2html(str.to_string());

        // assert
        assert_eq!(expected, actual);
    }
}
