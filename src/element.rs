use std::{borrow::Cow, collections::HashMap, fmt::Display, io::Cursor, string::FromUtf8Error};

use quick_xml::{events::{BytesEnd, BytesStart, Event}, Writer};

use crate::{traits::GetEvents, util::{get_attribute, get_attributes, qname_to_string, set_attribute}, EmptyElement, Error, Item, Tag};

/** An XML element: ```<element></element>``` */
#[derive(Debug, Clone)]
pub struct Element<'a> {
    pub(crate) start: BytesStart<'a>,
    pub(crate) end: BytesEnd<'a>,
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
    let Item::Element(element) = &parse(&xml)?[0] else {
        panic!();
    };

    let a_elements = element.find_descendants(&|item| {
        let Item::Element(el) = item else {
            return false;
        };
        el.get_name().unwrap() == "a"
    });

    assert_eq!(a_elements.len(), 2);
    # Ok::<(), Error>(())
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