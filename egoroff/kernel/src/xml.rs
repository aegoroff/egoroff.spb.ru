use anyhow::Result;
use quick_xml::{
    Writer,
    events::{BytesEnd, BytesStart, BytesText, Event},
};
use std::io::Cursor;

pub struct Builder {
    writer: Writer<Cursor<Vec<u8>>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            writer: Writer::new(Cursor::new(Vec::new())),
        }
    }

    pub fn to_string(self) -> Result<String> {
        let mut result = self.writer.into_inner().into_inner();

        let mut xml: Vec<u8> = Vec::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>");
        xml.append(&mut result);
        let result = String::from_utf8(xml)?;
        Ok(result)
    }

    pub fn write_text(&mut self, txt: &str) -> Result<(), anyhow::Error> {
        let txt = BytesText::new(txt);
        self.writer.write_event(Event::Text(txt))?;
        Ok(())
    }

    pub fn write_start_tag(&mut self, elt: &str) -> Result<(), anyhow::Error> {
        let start_tag = BytesStart::new(elt);
        self.writer.write_event(Event::Start(start_tag))?;
        Ok(())
    }

    pub fn write_attributed_start_tag<'a, I: Iterator<Item = (&'a str, &'a str)>>(
        &mut self,
        elt: &str,
        attributes: I,
    ) -> Result<(), anyhow::Error> {
        let mut start_tag = BytesStart::new(elt);
        for (attr, val) in attributes {
            start_tag.push_attribute((attr, val));
        }
        self.writer.write_event(Event::Start(start_tag))?;
        Ok(())
    }

    pub fn write_end_tag(&mut self, elt: &str) -> Result<(), anyhow::Error> {
        self.writer.write_event(Event::End(BytesEnd::new(elt)))?;
        Ok(())
    }

    pub fn write_element(&mut self, elt: &str, txt: &str) -> Result<(), anyhow::Error> {
        self.write_start_tag(elt)?;

        self.write_text(txt)?;

        self.write_end_tag(elt)?;
        Ok(())
    }

    pub fn write_attributed_element<'a, I: Iterator<Item = (&'a str, &'a str)>>(
        &mut self,
        elt: &str,
        txt: &str,
        attributes: I,
    ) -> Result<(), anyhow::Error> {
        self.write_attributed_start_tag(elt, attributes)?;

        self.write_text(txt)?;

        self.write_end_tag(elt)?;
        Ok(())
    }

    pub fn write_empty_attributed_element<'a, I: Iterator<Item = (&'a str, &'a str)>>(
        &mut self,
        elt: &str,
        attributes: I,
    ) -> Result<(), anyhow::Error> {
        self.write_attributed_start_tag(elt, attributes)?;

        self.write_end_tag(elt)?;
        Ok(())
    }
}
