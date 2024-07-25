use std::{collections::HashMap, fmt::Display, io::Cursor, ops::Deref, string::FromUtf8Error};

use quick_xml::{
    events::{
        attributes::Attribute, BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText,
        Event,
    },
    name::QName,
    Error, Reader, Writer,
};

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

pub trait Item {
    fn get_all_events(&self) -> Vec<Event>;
}

pub trait Elem {
    fn get_attributes(&self) -> Result<HashMap<String, String>, FromUtf8Error>;
    fn get_attribute(&self, key: &str) -> Result<Option<String>, Error>;
    fn set_attribute(&mut self, key: &str, value: &str) -> Result<(), FromUtf8Error>;
}

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

pub struct Element<'a> {
    start: BytesStart<'a>,
    end: BytesEnd<'a>,
    pub children: Vec<XmlItem<'a>>,
}

impl<'a> Element<'a> {
    pub fn get_name(&self) -> Result<String, FromUtf8Error> {
        qname_to_string(&self.start.name())
    }

    pub fn set_name(&mut self, name: &str) {
        self.start.set_name(name.as_bytes());
    }

    pub fn new(name: &'a str) -> Self {
        let start = BytesStart::new(name);
        let end = BytesEnd::new(name);
        Element {
            start,
            end,
            children: Vec::new(),
        }
    }
}

impl Item for Element<'_> {
    fn get_all_events(&self) -> Vec<Event> {
        let mut events = vec![Event::Start(self.start.to_owned())];

        for child in &self.children {
            let mut child_events = match child {
                XmlItem::Element(element) => element.get_all_events(),
                XmlItem::EmptyElement(element) => element.get_all_events(),
                XmlItem::Comment(text) => text.get_all_events(),
                XmlItem::Text(text) => text.get_all_events(),
                XmlItem::DocType(text) => text.get_all_events(),
                XmlItem::CData(text) => text.get_all_events(),
                XmlItem::Decl(text) => text.get_all_events(),
                XmlItem::PI(text) => text.get_all_events(),
            };

            events.append(&mut child_events);
        }

        events.push(Event::End(self.end.to_owned()));

        events
    }
}

impl Elem for Element<'_> {
    fn get_attributes(&self) -> Result<HashMap<String, String>, FromUtf8Error> {
        get_attributes(&self.start)
    }

    fn get_attribute(&self, key: &str) -> Result<Option<String>, Error> {
        get_attribute(&self.start, key)
    }

    fn set_attribute(&mut self, key: &str, value: &str) -> Result<(), FromUtf8Error> {
        set_attribute(&mut self.start, key, value)
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

pub struct EmptyElement<'a> {
    element: BytesStart<'a>,
}

impl<'a> EmptyElement<'a> {
    pub fn get_name(&self) -> Result<String, FromUtf8Error> {
        qname_to_string(&self.element.name())
    }

    pub fn set_name(&mut self, name: &str) {
        self.element.set_name(name.as_bytes());
    }
}

impl Item for EmptyElement<'_> {
    fn get_all_events(&self) -> Vec<Event> {
        vec![Event::Empty(self.element.to_owned())]
    }
}

impl Elem for EmptyElement<'_> {
    fn get_attributes(&self) -> Result<HashMap<String, String>, FromUtf8Error> {
        get_attributes(&self.element)
    }

    fn get_attribute(&self, key: &str) -> Result<Option<String>, Error> {
        get_attribute(&self.element, key)
    }

    fn set_attribute(&mut self, key: &str, value: &str) -> Result<(), FromUtf8Error> {
        set_attribute(&mut self.element, key, value)
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

pub enum OtherItem<'a> {
    Comment(BytesText<'a>),
    Text(BytesText<'a>),
    DocType(BytesText<'a>),
    CData(BytesCData<'a>),
    Decl(BytesDecl<'a>),
    PI(BytesPI<'a>),
}

pub struct Other<'a> {
    item: OtherItem<'a>,
}

impl<'a> Other<'a> {
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

impl Item for Other<'_> {
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

pub fn parse_trimmed(xml: &str) -> Result<Vec<XmlItem>, quick_xml::Error> {
    let events = get_all_events(xml, true)?;
    Ok(parse_events(events))
}

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
