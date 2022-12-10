use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    str,
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

    let parents: HashSet<&[u8]> = PARENTS.iter().copied().collect();
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

                let original_attributes = e.attributes().filter_map(|attr| attr.ok());
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
                            "hame" => format!("/blog/{}.html", val),
                            _ => "".to_string(),
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
            Ok(Event::CData(e)) => {
                let escaped = e.escape().unwrap();
                let evt = Event::Text(escaped);
                assert!(writer.write_event(evt).is_ok());
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
    fn converter_tests(#[case] str: &str, #[case] expected: &str) {
        // arrange

        // act
        let actual = xml2html(str.to_string());

        // assert
        assert_eq!(expected, actual);
    }
}
