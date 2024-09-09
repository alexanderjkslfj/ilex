use std::fmt::Display;

use quick_xml::events::Event;

use crate::{util::GetEvents, Element, Other};

/** Any XML item. */
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<'a> {
    /** Element ```<tag attr="value">...</tag>``` or ```<tag attr="value" />```. */
    Element(Element<'a>),
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
    /** Create a new Element. */
    pub fn new_element(name: &'a str, self_closing: bool) -> Self {
        Item::Element(Element::new(name, self_closing))
    }

    /** Create a new comment item. */
    pub fn new_comment(content: &'a str) -> Self {
        Item::Comment(Other::new_comment(content))
    }

    /** Create a new text item. */
    pub fn new_text(content: &'a str) -> Self {
        Item::Text(Other::new_text(content))
    }

    /** Create a new doctype item. */
    pub fn new_doctype(content: &'a str) -> Self {
        Item::DocType(Other::new_doctype(content))
    }

    /** Create a new character data item. */
    pub fn new_cdata(content: &'a str) -> Self {
        Item::CData(Other::new_cdata(content))
    }

    /** Create a new declaration item. */
    pub fn new_decl(version: &str, encoding: Option<&str>, standalone: Option<&str>) -> Self {
        Item::Decl(Other::new_decl(version, encoding, standalone))
    }

    /** Create a new processing instruction item. */
    pub fn new_pi(content: &'a str) -> Self {
        Item::PI(Other::new_pi(content))
    }
}

impl Display for Item<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Element(element) => element.fmt(f),
            Item::Comment(comment) => comment.fmt(f),
            Item::Text(text) => text.fmt(f),
            Item::DocType(doctype) => doctype.fmt(f),
            Item::CData(cdata) => cdata.fmt(f),
            Item::Decl(decl) => decl.fmt(f),
            Item::PI(pi) => pi.fmt(f),
        }
    }
}

impl GetEvents for Item<'_> {
    fn get_all_events(&self) -> Box<dyn Iterator<Item = Event> + '_> {
        match self {
            Item::Element(element) => element.get_all_events(),
            Item::Comment(comment) => comment.get_all_events(),
            Item::Text(text) => text.get_all_events(),
            Item::DocType(doctype) => doctype.get_all_events(),
            Item::CData(cdata) => cdata.get_all_events(),
            Item::Decl(decl) => decl.get_all_events(),
            Item::PI(pi) => pi.get_all_events(),
        }
    }
}
