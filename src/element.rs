use std::{borrow::Cow, collections::HashMap, fmt::Display, io::Cursor, string::FromUtf8Error};

use quick_xml::{
    events::{BytesEnd, BytesStart, Event},
    Writer,
};

use crate::{
    traits::GetEvents,
    util::{get_attribute, get_attributes, qname_to_string, set_attribute},
    EmptyElement, Error, Item, Tag,
};

/** An XML element: ```<element></element>``` */
#[derive(Debug, Clone, PartialEq, Eq)]
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

    assert_eq!(a_elements.count(), 2);
    # Ok::<(), Error>(())
    ```*/
    pub fn find_descendants(
        &self,
        predicate: &'a impl Fn(&Item) -> bool,
    ) -> Box<dyn Iterator<Item = &Item> + '_> {
        // get direct children matching the predicate
        let matching_children = self.children.iter().filter(|item| predicate(item));

        // get deeper descendants matching the predicate
        let matching_descendants = self
            .children
            .iter()
            // select only the children which are elements (and can therefore have deeper descendants)
            .filter_map(|child| match child {
                Item::Element(element) => Some(element),
                _ => None,
            })
            // get the children's descendants matching the predicate (recursively)
            .flat_map(|child| child.find_descendants(predicate));

        let chain = Iterator::chain(matching_children, matching_descendants);

        Box::new(chain)
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
    pub fn get_items_at_depth(&self, depth: usize) -> Box<dyn Iterator<Item = &Item> + '_> {
        if depth == 1 {
            return Box::new(self.children.iter());
        }
        if depth == 0 {
            panic!("depth cannot be zero.");
        }

        let items = self
            .children
            .iter()
            // select only the children which are elements (and can therefore go deeper)
            .filter_map(|item| match item {
                Item::Element(element) => Some(element),
                _ => None,
            })
            // get the deeper items (recursively)
            .flat_map(move |element| element.get_items_at_depth(depth - 1));

        Box::new(items)
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
    fn get_all_events(&self) -> Box<dyn Iterator<Item = Event> + '_> {
        let start_event = std::iter::once(Event::Start(self.start.to_owned()));
        let end_event = std::iter::once(Event::End(self.end.to_owned()));

        let child_events = self
            .children
            .iter()
            .flat_map(|child| child.get_all_events());

        let events = start_event.chain(child_events.chain(end_event));

        Box::new(events)
    }
}
