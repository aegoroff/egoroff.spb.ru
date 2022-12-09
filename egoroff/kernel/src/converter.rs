use std::{collections::HashMap, io::Cursor};

use quick_xml::{
    events::{BytesEnd, BytesStart, Event},
    Reader, Writer,
};

const REPLACES: &[(&[u8], &str)] = &[
    (b"example", "pre"),
    (b"quote", "blockquote"),
    (b"link", "a"),
    (b"center", "div"),
    // ("div1", "2"),
    // ("div2", "3"),
    // ("div3", "4"),
    // ("head", "h"),
];

pub fn xml2html(input: String) -> String {
    if !input.starts_with("<?xml version=\"1.0\"?>") {
        return input;
    }
    let mut reader = Reader::from_str(&input);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let replaces: HashMap<&[u8], &str> = REPLACES.iter().map(|(k, v)| (*k, *v)).collect();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if replaces.contains_key(e.name().as_ref()) => {
                let replace = replaces.get(e.name().as_ref()).unwrap();
                let mut elem = BytesStart::new(*replace);

                elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));

                assert!(writer.write_event(Event::Start(elem)).is_ok());
            }
            Ok(Event::End(e)) if replaces.contains_key(e.name().as_ref()) => {
                let replace = replaces.get(e.name().as_ref()).unwrap();
                let elem = BytesEnd::new(*replace);
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
    fn converter_tests(#[case] str: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = xml2html(str.to_string());

        // assert
        assert_eq!(expected, actual);
    }
}
