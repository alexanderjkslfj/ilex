use std::{collections::HashMap, fmt::Display, io::Cursor, ops::Deref, string::FromUtf8Error};

use quick_xml::{
    events::{
        attributes::Attribute, BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText,
        Event,
    },
    name::QName,
    Error, Reader, Writer,
};

/** Any XML item, such as an element ```<element></element>```, a comment ```<!-- comment -->``` or even just some text ```text```. */
pub enum XmlItem<'a> {
    /** Element ```<tag attr="value">...</tag>```. */
    Element(Element<'a>),
    /** Empty element ```<tag attr="value" />```. */
    EmptyElement(EmptyElement<'a>),
    /** Comment ```<!-- ... -->```. */
    Comment(Other<'a>),
    /** Escaped character data between tags. */
    Text(Other<'a>),
    /** Document type definition data (DTD) stored in ```<!DOCTYPE ...>```. */
    DocType(Other<'a>),
    /** Unescaped character data stored in ```<![CDATA[...]]>```. */
    CData(Other<'a>),
    /** XML declaration ```<?xml ...?>```. */
    Decl(Other<'a>),
    /** Processing instruction ```<?...?>```. */
    PI(Other<'a>),
}

impl XmlItem<'_> {
    fn get_all_events(&self) -> Vec<Event> {
        match self {
            XmlItem::Element(element) => element.get_all_events(),
            XmlItem::EmptyElement(element) => element.get_all_events(),
            XmlItem::Comment(comment) => comment.get_all_events(),
            XmlItem::Text(text) => text.get_all_events(),
            XmlItem::DocType(doctype) => doctype.get_all_events(),
            XmlItem::CData(cdata) => cdata.get_all_events(),
            XmlItem::Decl(decl) => decl.get_all_events(),
            XmlItem::PI(pi) => pi.get_all_events(),
        }
    }
}

trait GetEvents {
    fn get_all_events(&self) -> Vec<Event>;
}

/** Used for managing the tag and attributes of an Element or EmptyElement. */
pub trait Elem<'a> {
    /** Get a map of all attributes. If an attribute occurs multiple times, the last occurence is used. */
    fn get_attributes(&self) -> Result<HashMap<String, String>, FromUtf8Error>;
    /** Get an attribute. */
    fn get_attribute(&self, key: &str) -> Result<Option<String>, Error>;
    /** Add or replace an attribute. */
    fn set_attribute(&mut self, key: &str, value: &str) -> Result<(), FromUtf8Error>;
    /** Get the tag name. */
    fn get_name(&self) -> Result<String, FromUtf8Error>;
    /** Change the tag name. */
    fn set_name(&mut self, name: &'a str);
}

/** Stringify a list of XML items.

Equivalent to calling to_string on each item and concatenating the results.
*/
pub fn items_to_string(items: &[XmlItem]) -> String {
    let mut str = String::new();
    for item in items {
        let item_str = match &item {
            XmlItem::Text(text) => text.to_string(),
            XmlItem::Comment(text) => text.to_string(),
            XmlItem::CData(text) => text.to_string(),
            XmlItem::PI(text) => text.to_string(),
            XmlItem::Decl(text) => text.to_string(),
            XmlItem::DocType(text) => text.to_string(),
            XmlItem::Element(text) => text.to_string(),
            XmlItem::EmptyElement(text) => text.to_string(),
        };
        str.push_str(&item_str);
    }
    str
}

/** An XML element ```<element></element>``` */
pub struct Element<'a> {
    start: BytesStart<'a>,
    end: BytesEnd<'a>,
    /** All items contained within the element. */
    pub children: Vec<XmlItem<'a>>,
}

impl<'a> Element<'a> {
    pub fn new(name: &'a str) -> Self {
        let start = BytesStart::new(name);
        let end = BytesEnd::new(name);
        Element {
            start,
            end,
            children: Vec::new(),
        }
    }

    /** Get all items at a certain depth within the element.
    ```xml
    <element>
        <item depth="1">
            <item at-depth="2">
                This text is at depth 3.
            </item>
        </item>
    </element>
    ```*/
    pub fn get_items_at_depth(&self, depth: usize) -> Vec<&XmlItem> {
        if depth == 0 {
            panic!("depth cannot be zero.");
        }
        if depth == 1 {
            return Vec::from_iter(self.children.iter());
        }

        let mut items = Vec::new();

        for child in &self.children {
            let XmlItem::Element(element) = child else {
                continue;
            };
            items.append(&mut element.get_items_at_depth(depth - 1));
        }

        items
    }

    /** Get the text content of all text items within the element.
    ```xml
    <element>Hello<child>World</child></element>
    ```
    The above would result in "HelloWorld".
         */
    pub fn get_text_content(&self) -> Result<String, FromUtf8Error> {
        let mut content = String::new();

        for child in &self.children {
            match child {
                XmlItem::Text(text) => {
                    content.push_str(&text.get_value()?);
                }
                XmlItem::Element(element) => {
                    content.push_str(&element.get_text_content()?);
                }
                _ => (),
            }
        }

        Ok(content)
    }
}

impl GetEvents for Element<'_> {
    fn get_all_events(&self) -> Vec<Event> {
        let mut events = vec![Event::Start(self.start.to_owned())];

        for child in &self.children {
            events.append(&mut child.get_all_events());
        }

        events.push(Event::End(self.end.to_owned()));

        events
    }
}

impl<'a> Elem<'a> for Element<'a> {
    fn get_attributes(&self) -> Result<HashMap<String, String>, FromUtf8Error> {
        get_attributes(&self.start)
    }

    fn get_attribute(&self, key: &str) -> Result<Option<String>, Error> {
        get_attribute(&self.start, key)
    }

    fn set_attribute(&mut self, key: &str, value: &str) -> Result<(), FromUtf8Error> {
        set_attribute(&mut self.start, key, value)
    }

    fn set_name(&mut self, name: &'a str) {
        self.start.set_name(name.as_bytes());
        self.end = BytesEnd::new(name); // TODO: do it without replacing the entire object
    }

    fn get_name(&self) -> Result<String, FromUtf8Error> {
        qname_to_string(&self.start.name())
    }
}

impl Display for Element<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        for event in self.get_all_events() {
            writer.write_event(event).unwrap();
        }

        let result = String::from_utf8(writer.into_inner().into_inner()).unwrap();

        write!(f, "{result}")
    }
}

/** A self-closing XML element ```<element />```. */
pub struct EmptyElement<'a> {
    element: BytesStart<'a>,
}

impl<'a> EmptyElement<'a> {
    pub fn new(name: &'a str) -> Self {
        EmptyElement {
            element: BytesStart::new(name),
        }
    }
}

impl GetEvents for EmptyElement<'_> {
    fn get_all_events(&self) -> Vec<Event> {
        vec![Event::Empty(self.element.to_owned())]
    }
}

impl<'a> Elem<'a> for EmptyElement<'a> {
    fn get_attributes(&self) -> Result<HashMap<String, String>, FromUtf8Error> {
        get_attributes(&self.element)
    }

    fn get_attribute(&self, key: &str) -> Result<Option<String>, Error> {
        get_attribute(&self.element, key)
    }

    fn set_attribute(&mut self, key: &str, value: &str) -> Result<(), FromUtf8Error> {
        set_attribute(&mut self.element, key, value)
    }

    fn get_name(&self) -> Result<String, FromUtf8Error> {
        qname_to_string(&self.element.name())
    }

    fn set_name(&mut self, name: &str) {
        self.element.set_name(name.as_bytes());
    }
}

impl Display for EmptyElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let event = Event::Empty(self.element.to_owned());
        writer.write_event(event).unwrap();
        let result = String::from_utf8(writer.into_inner().into_inner()).unwrap();
        write!(f, "{result}")
    }
}

fn get_attributes(start: &BytesStart) -> Result<HashMap<String, String>, FromUtf8Error> {
    let attrs: Vec<Attribute> = start
        .attributes()
        .filter_map(|attr| {
            if attr.is_ok() {
                Some(attr.unwrap())
            } else {
                None
            }
        })
        .collect();

    let mut attributes = HashMap::with_capacity(attrs.len());

    for attr in attrs {
        let key = qname_to_string(&attr.key)?;
        let value = String::from_utf8(attr.value.deref().to_vec())?;
        attributes.insert(key, value);
    }

    Ok(attributes)
}

fn get_attribute(start: &BytesStart, key: &str) -> Result<Option<String>, Error> {
    let Some(attr) = start.try_get_attribute(key)? else {
        return Ok(None);
    };
    let value_res = u8_to_string(&attr.value);
    if value_res.is_err() {
        return Err(Error::NonDecodable(Some(
            value_res.unwrap_err().utf8_error(),
        )));
    }
    Ok(Some(value_res.unwrap()))
}

fn set_attribute(start: &mut BytesStart, key: &str, value: &str) -> Result<(), FromUtf8Error> {
    let mut attributes = get_attributes(start)?;
    attributes.insert(String::from(key), String::from(value));
    let attrs = attributes
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()));
    start.clear_attributes();
    start.extend_attributes(attrs);
    Ok(())
}

/** Any XML item that is not an element. */
pub enum OtherItem<'a> {
    Comment(BytesText<'a>),
    Text(BytesText<'a>),
    DocType(BytesText<'a>),
    CData(BytesCData<'a>),
    Decl(BytesDecl<'a>),
    PI(BytesPI<'a>),
}

/** Wrapper for any XML item that is not an element. */
pub struct Other<'a> {
    item: OtherItem<'a>,
}

impl<'a> Other<'a> {
    pub fn new(item: OtherItem<'a>) -> Self {
        Other { item }
    }

    pub fn new_comment(content: &'a str) -> Self {
        Other {
            item: OtherItem::Comment(BytesText::new(content)),
        }
    }

    pub fn new_text(content: &'a str) -> Self {
        Other {
            item: OtherItem::Text(BytesText::new(content)),
        }
    }

    pub fn new_doctype(content: &'a str) -> Self {
        Other {
            item: OtherItem::DocType(BytesText::new(content)),
        }
    }

    pub fn new_cdata(content: &'a str) -> Self {
        Other {
            item: OtherItem::CData(BytesCData::new(content)),
        }
    }

    pub fn new_decl(version: &str, encoding: Option<&str>, standalone: Option<&str>) -> Self {
        Other {
            item: OtherItem::Decl(BytesDecl::new(version, encoding, standalone)),
        }
    }

    pub fn new_pi(content: &'a str) -> Self {
        Other {
            item: OtherItem::PI(BytesPI::new(content)),
        }
    }

    /** Change the value of an item.
    ```rust
        use ilex::{Other, OtherItem};
        use quick_xml::events::BytesText;

        let mut text_item = Other::new(OtherItem::Text(BytesText::new("hello")));
        text_item.set_value("world");
        assert_eq!("world", text_item.to_string());
    ```*/
    pub fn set_value<'b: 'a>(&mut self, value: &'b str) {
        self.item = match &self.item {
            OtherItem::Comment(_) => OtherItem::Comment(BytesText::new(value)),
            OtherItem::Text(_) => OtherItem::Text(BytesText::new(value)),
            OtherItem::DocType(_) => OtherItem::DocType(BytesText::new(value)),
            OtherItem::CData(_) => OtherItem::CData(BytesCData::new(value)),
            OtherItem::Decl(_) => OtherItem::Decl(BytesDecl::new("1.0", None, None)), // TODO: improve
            OtherItem::PI(_) => OtherItem::PI(BytesPI::new(value)),
        };
    }

    /** Get the value of an item.
    ```rust
        use ilex::{Other, OtherItem};
        use quick_xml::events::BytesText;

        let text_item = Other::new(OtherItem::Text(BytesText::new("hello world")));
        assert_eq!("hello world", text_item.get_value().unwrap());
    ```*/
    pub fn get_value(&self) -> Result<String, FromUtf8Error> {
        match &self.item {
            OtherItem::Comment(event) => u8_to_string(event),
            OtherItem::Text(event) => u8_to_string(event),
            OtherItem::DocType(event) => u8_to_string(event),
            OtherItem::CData(event) => u8_to_string(event),
            OtherItem::Decl(event) => u8_to_string(event),
            OtherItem::PI(event) => u8_to_string(event),
        }
    }

    fn get_event(&self) -> Event {
        match &self.item {
            OtherItem::Comment(event) => Event::Comment(event.to_owned()),
            OtherItem::Text(event) => Event::Text(event.to_owned()),
            OtherItem::DocType(event) => Event::DocType(event.to_owned()),
            OtherItem::CData(event) => Event::CData(event.to_owned()),
            OtherItem::Decl(event) => Event::Decl(event.to_owned()),
            OtherItem::PI(event) => Event::PI(event.to_owned()),
        }
    }
}

impl GetEvents for Other<'_> {
    fn get_all_events(&self) -> Vec<Event> {
        vec![self.get_event()]
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

/** Parse raw XML and trim whitespace at the beginning and end of text. */
pub fn parse_trimmed(xml: &str) -> Result<Vec<XmlItem>, quick_xml::Error> {
    let events = get_all_events(xml, true)?;
    Ok(parse_events(events))
}

/** Parse raw XML. */
pub fn parse(xml: &str) -> Result<Vec<XmlItem>, quick_xml::Error> {
    let events = get_all_events(xml, false)?;
    Ok(parse_events(events))
}

fn parse_events(events: Vec<Event>) -> Vec<XmlItem> {
    let mut items = Vec::new();

    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Text(item) => items.push(XmlItem::Text(Other {
                item: OtherItem::Text(item.to_owned()),
            })),
            Event::Comment(item) => items.push(XmlItem::Comment(Other {
                item: OtherItem::Comment(item.to_owned()),
            })),
            Event::CData(item) => items.push(XmlItem::CData(Other {
                item: OtherItem::CData(item.to_owned()),
            })),
            Event::PI(item) => items.push(XmlItem::PI(Other {
                item: OtherItem::PI(item.to_owned()),
            })),
            Event::Decl(item) => items.push(XmlItem::Decl(Other {
                item: OtherItem::Decl(item.to_owned()),
            })),
            Event::DocType(item) => items.push(XmlItem::DocType(Other {
                item: OtherItem::DocType(item.to_owned()),
            })),
            Event::Empty(item) => items.push(XmlItem::EmptyElement(EmptyElement {
                element: item.to_owned(),
            })),
            Event::Start(start) => {
                let mut depth = 1;
                let mut sub_events = Vec::new();
                let end = loop {
                    i += 1;
                    let event = &events[i];
                    match event {
                        Event::Start(_) => {
                            depth += 1;
                        }
                        Event::End(end) => {
                            depth -= 1;
                            if depth == 0 {
                                break end.to_owned();
                            }
                        }
                        _ => (),
                    }
                    sub_events.push(event.to_owned());
                };
                items.push(XmlItem::Element(Element {
                    start: start.to_owned(),
                    end,
                    children: parse_events(sub_events),
                }));
            }
            Event::End(_) => panic!("aaaaa!"),
            Event::Eof => break,
        }
        i += 1;
    }

    items
}

fn get_all_events(xml: &str, trim: bool) -> Result<Vec<Event>, quick_xml::Error> {
    let mut events = Vec::new();

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(trim);

    loop {
        match reader.read_event() {
            Err(err) => return Err(err),

            Ok(Event::Eof) => break,

            Ok(e) => events.push(e),
        };
    }

    Ok(events)
}

fn qname_to_string(qname: &QName) -> Result<String, FromUtf8Error> {
    u8_to_string(qname.as_ref())
}

fn u8_to_string(u8: &[u8]) -> Result<String, FromUtf8Error> {
    String::from_utf8(u8.to_vec())
}
