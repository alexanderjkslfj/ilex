use std::fmt::Display;

use quick_xml::events::Event;

use crate::{traits::GetEvents, Element, EmptyElement, Other};

/** Any XML item. */
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl GetEvents for Item<'_> {
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
}