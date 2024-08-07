use std::{
    borrow::Cow, collections::HashMap, fmt::Display, io::Cursor, ops::Deref, string::FromUtf8Error,
};

use quick_xml::{
    events::{
        attributes::Attribute, BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText,
        Event,
    },
    name::QName,
    Error, Reader, Writer,
};

/** Any XML item. */
#[derive(Debug, Clone)]
pub enum Item<'a> {
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

impl<'a> Item<'a> {
    fn get_all_events(&self) -> Vec<Event> {
        match self {
            Item::Element(element) => element.get_all_events(),
            Item::EmptyElement(element) => element.get_all_events(),
            Item::Comment(comment) => comment.get_all_events(),
            Item::Text(text) => text.get_all_events(),
            Item::DocType(doctype) => doctype.get_all_events(),
            Item::CData(cdata) => cdata.get_all_events(),
            Item::Decl(decl) => decl.get_all_events(),
            Item::PI(pi) => pi.get_all_events(),
        }
    }

    pub fn new_element(name: &'a str) -> Self {
        Item::Element(Element::new(name))
    }

    pub fn new_empty_element(name: &'a str) -> Self {
        Item::EmptyElement(EmptyElement::new(name))
    }

    pub fn new_comment(content: &'a str) -> Self {
        Item::Comment(Other::new_comment(content))
    }

    pub fn new_text(content: &'a str) -> Self {
        Item::Text(Other::new_text(content))
    }

    pub fn new_doctype(content: &'a str) -> Self {
        Item::DocType(Other::new_doctype(content))
    }

    pub fn new_cdata(content: &'a str) -> Self {
        Item::CData(Other::new_cdata(content))
    }

    pub fn new_decl(version: &str, encoding: Option<&str>, standalone: Option<&str>) -> Self {
        Item::Decl(Other::new_decl(version, encoding, standalone))
    }

    pub fn new_pi(content: &'a str) -> Self {
        Item::PI(Other::new_pi(content))
    }
}

impl Display for Item<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Element(element) => element.fmt(f),
            Item::EmptyElement(element) => element.fmt(f),
            Item::Comment(comment) => comment.fmt(f),
            Item::Text(text) => text.fmt(f),
            Item::DocType(doctype) => doctype.fmt(f),
            Item::CData(cdata) => cdata.fmt(f),
            Item::Decl(decl) => decl.fmt(f),
            Item::PI(pi) => pi.fmt(f),
        }
    }
}

trait GetEvents {
    fn get_all_events(&self) -> Vec<Event>;
}

/** Used for accessing the name and attributes of an Element or EmptyElement. */
pub trait Tag<'a> {
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

Equivalent to calling ```to_string``` on each item and concatenating the results.
*/
pub fn items_to_string(items: &[Item]) -> String {
    let mut str = String::new();
    for item in items {
        let item_str = match &item {
            Item::Text(text) => text.to_string(),
            Item::Comment(text) => text.to_string(),
            Item::CData(text) => text.to_string(),
            Item::PI(text) => text.to_string(),
            Item::Decl(text) => text.to_string(),
            Item::DocType(text) => text.to_string(),
            Item::Element(text) => text.to_string(),
            Item::EmptyElement(text) => text.to_string(),
        };
        str.push_str(&item_str);
    }
    str
}

/** An XML element: ```<element></element>``` */
#[derive(Debug, Clone)]
pub struct Element<'a> {
    start: BytesStart<'a>,
    end: BytesEnd<'a>,
    /** All items contained within the element. */
    pub children: Vec<Item<'a>>,
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

    /** Get all descendants matching the predicate.
    ```rust
    // Example of finding all elements with tag name "a":
    let xml = "<element><a></a><b><a></a></b><c>text</c></element>";

    # use ilex_xml::*;
    let Item::Element(element) = &parse(&xml).unwrap()[0] else {
        panic!();
    };

    let a_elements = element.find_descendants(&|item| {
        let Item::Element(el) = item else {
            return false;
        };
        el.get_name().unwrap() == "a"
    });

    assert_eq!(a_elements.len(), 2);
    ```*/
    pub fn find_descendants(&self, predicate: &impl Fn(&Item) -> bool) -> Vec<&Item> {
        println!("{}", self.get_name().unwrap());

        let mut result: Vec<&Item<'_>> = self
            .children
            .iter()
            .filter(|item| predicate(item))
            .collect();

        for child in &self.children {
            let Item::Element(element) = child else {
                continue;
            };
            result.append(&mut element.find_descendants(predicate));
        }

        result
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
    pub fn get_items_at_depth(&self, depth: usize) -> Vec<&Item> {
        if depth == 0 {
            panic!("depth cannot be zero.");
        }
        if depth == 1 {
            return Vec::from_iter(self.children.iter());
        }

        let mut items = Vec::new();

        for child in &self.children {
            let Item::Element(element) = child else {
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
                Item::Text(text) => {
                    content.push_str(&text.get_value()?);
                }
                Item::Element(element) => {
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

impl<'a> Tag<'a> for Element<'a> {
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

impl<'a> From<EmptyElement<'a>> for Element<'a> {
    fn from(value: EmptyElement<'a>) -> Self {
        let name = value.get_name().unwrap();
        Element {
            start: value.element,
            end: BytesEnd::new(Cow::from(name)),
            children: Vec::new(),
        }
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

/** A self-closing XML element: ```<element />``` */
#[derive(Debug, Clone)]
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

impl<'a> Tag<'a> for EmptyElement<'a> {
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
#[derive(Debug, Clone)]
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
        let value = comment.get_value().unwrap();
        assert_eq!(value, "hello world");
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
pub fn parse_trimmed(xml: &str) -> Result<Vec<Item>, quick_xml::Error> {
    let events = get_all_events(xml, true)?;
    Ok(parse_events(events))
}

/** Parse raw XML. */
pub fn parse(xml: &str) -> Result<Vec<Item>, quick_xml::Error> {
    let events = get_all_events(xml, false)?;
    Ok(parse_events(events))
}

fn parse_events(events: Vec<Event>) -> Vec<Item> {
    let mut items = Vec::new();

    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Text(item) => items.push(Item::Text(Other::Text(item.to_owned()))),
            Event::Comment(item) => items.push(Item::Comment(Other::Comment(item.to_owned()))),
            Event::CData(item) => items.push(Item::CData(Other::CData(item.to_owned()))),
            Event::PI(item) => items.push(Item::PI(Other::PI(item.to_owned()))),
            Event::Decl(item) => items.push(Item::Decl(Other::Decl(item.to_owned()))),
            Event::DocType(item) => items.push(Item::DocType(Other::DocType(item.to_owned()))),
            Event::Empty(item) => items.push(Item::EmptyElement(EmptyElement {
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
                items.push(Item::Element(Element {
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
