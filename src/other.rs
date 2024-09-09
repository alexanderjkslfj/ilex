use std::{fmt::Display, io::Cursor, string::FromUtf8Error};

use quick_xml::{
    events::{BytesCData, BytesDecl, BytesPI, BytesText, Event},
    Writer,
};

use crate::{traits::GetEvents, util::u8_to_string};

/** Any XML item that is not an element. */
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Other<'a> {
    Comment(BytesText<'a>),
    Text(BytesText<'a>),
    DocType(BytesText<'a>),
    CData(BytesCData<'a>),
    Decl(BytesDecl<'a>),
    PI(BytesPI<'a>),
}

impl<'a> Other<'a> {
    pub fn new_comment(content: &'a str) -> Self {
        Other::Comment(BytesText::new(content))
    }

    pub fn new_text(content: &'a str) -> Self {
        Other::Text(BytesText::new(content))
    }

    pub fn new_doctype(content: &'a str) -> Self {
        Other::DocType(BytesText::new(content))
    }

    pub fn new_cdata(content: &'a str) -> Self {
        Other::CData(BytesCData::new(content))
    }

    pub fn new_pi(content: &'a str) -> Self {
        Other::PI(BytesPI::new(content))
    }

    pub fn new_decl(version: &str, encoding: Option<&str>, standalone: Option<&str>) -> Self {
        Other::Decl(BytesDecl::new(version, encoding, standalone))
    }

    /** Get the value of an item.
    ```rust
        # use ilex_xml::Other;
        let comment = Other::new_comment("hello world");
        let value = comment.get_value()?;
        assert_eq!(value, "hello world");
        # Ok::<(), std::string::FromUtf8Error>(())
    ```*/
    pub fn get_value(&self) -> Result<String, FromUtf8Error> {
        match &self {
            Other::Comment(event) => u8_to_string(event),
            Other::Text(event) => u8_to_string(event),
            Other::DocType(event) => u8_to_string(event),
            Other::CData(event) => u8_to_string(event),
            Other::Decl(event) => u8_to_string(event),
            Other::PI(event) => u8_to_string(event),
        }
    }

    fn get_event(&self) -> Event {
        match &self {
            Other::Comment(event) => Event::Comment(event.to_owned()),
            Other::Text(event) => Event::Text(event.to_owned()),
            Other::DocType(event) => Event::DocType(event.to_owned()),
            Other::CData(event) => Event::CData(event.to_owned()),
            Other::Decl(event) => Event::Decl(event.to_owned()),
            Other::PI(event) => Event::PI(event.to_owned()),
        }
    }
}

impl Display for Other<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let event = self.get_event();
        writer.write_event(event).unwrap();
        let result = String::from_utf8(writer.into_inner().into_inner()).unwrap();
        write!(f, "{result}")
    }
}

impl GetEvents for Other<'_> {
    fn get_all_events(&self) -> Box<dyn Iterator<Item = Event> + '_> {
        Box::new(std::iter::once(self.get_event()))
    }
}
