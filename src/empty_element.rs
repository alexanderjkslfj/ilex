use std::{collections::HashMap, fmt::Display, io::Cursor, string::FromUtf8Error};

use quick_xml::{
    events::{BytesStart, Event},
    Writer,
};

use crate::{
    traits::GetEvents,
    util::{get_attribute, get_attributes, qname_to_string, set_attribute},
    Element, Error, Tag,
};

/** A self-closing XML element: ```<element />``` */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyElement<'a> {
    pub(crate) element: BytesStart<'a>,
}

impl<'a> EmptyElement<'a> {
    pub fn new(name: &'a str) -> Self {
        EmptyElement {
            element: BytesStart::new(name),
        }
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

impl GetEvents for EmptyElement<'_> {
    fn get_all_events(&self) -> Box<dyn Iterator<Item = Event> + '_> {
        Box::new(std::iter::once(Event::Empty(self.element.to_owned())))
    }
}

impl<'a> TryFrom<Element<'a>> for EmptyElement<'a> {
    type Error = ();

    fn try_from(value: Element<'a>) -> Result<Self, Self::Error> {
        if value.children.is_empty() {
            Ok(EmptyElement {
                element: value.element,
            })
        } else {
            Err(())
        }
    }
}
