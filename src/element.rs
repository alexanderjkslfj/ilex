use std::{collections::HashMap, fmt::Display, io::Cursor, string::FromUtf8Error};

use quick_xml::{
    events::{BytesStart, Event},
    Writer,
};

use crate::{
    util::{qname_to_string, u8_to_string, GetEvents, ToStringSafe},
    Error, Item,
};

/** An XML element: ```<tag attr="value">...</tag>``` or ```<tag attr="value" />```. */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element<'a> {
    pub(crate) element: BytesStart<'a>,
    /** All items contained within the element. */
    pub children: Vec<Item<'a>>,
    /** If the element is childless: Should it be self-closing? */
    pub self_closing: bool,
}

impl<'a> Element<'a> {
    /** Create a new Element. */
    pub fn new(name: &'a str, self_closing: bool) -> Self {
        Element {
            element: BytesStart::new(name),
            children: Vec::new(),
            self_closing,
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

    /** Find all child elements with matching name */
    pub fn find_children(&'a self, name: &'a str) -> impl Iterator<Item = &Element> {
        self.children
            .iter()
            .filter_map(|child| match child {
                Item::Element(element) => Some(element),
                _ => None,
            })
            .filter(move |child| {
                let child_name = child.get_name();
                child_name.is_ok() && child_name.unwrap() == name
            })
    }

    /** Find all child elements with matching name */
    pub fn find_children_mut(&'a mut self, name: &'a str) -> impl Iterator<Item = &mut Element> {
        self.children
            .iter_mut()
            .filter_map(|child| match child {
                Item::Element(element) => Some(element),
                _ => None,
            })
            .filter(move |child| {
                let child_name = child.get_name();
                child_name.is_ok() && child_name.unwrap() == name
            })
    }

    /** Get all items at a certain depth within the element.

    Depth must not be zero.

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

    /** Get all items at a certain depth within the element.

    Depth must not be zero.

    ```xml
    <element>
        <item depth="1">
            <item at-depth="2">
                This text is at depth 3.
            </item>
        </item>
    </element>
    ```*/
    pub fn get_items_at_depth_mut(&'a mut self, depth: usize) -> Box<dyn Iterator<Item = &mut Item> + '_> {
        if depth == 1 {
            return Box::new(self.children.iter_mut());
        }
        if depth == 0 {
            panic!("depth cannot be zero.");
        }

        let items = self
            .children
            .iter_mut()
            // select only the children which are elements (and can therefore go deeper)
            .filter_map(|item| match item {
                Item::Element(element) => Some(element),
                _ => None,
            })
            // get the deeper items (recursively)
            .flat_map(move |element| element.get_items_at_depth_mut(depth - 1));

        Box::new(items)
    }

    /** Get the text content of all text items within the element.

    ```xml
    <element>Hello<child>World</child></element>
    ```

    The above would result in "HelloWorld".

    Parsing errors are silently ignored.*/
    pub fn get_text_content(&self) -> String {
        self.children
            .iter()
            .filter_map(|child| match child {
                Item::Text(text) => match text.get_value() {
                    Ok(text) => Some(text),
                    Err(_) => None,
                },
                Item::Element(element) => Some(element.get_text_content()),
                _ => None,
            })
            .collect()
    }

    /** Get all attributes.

    Parsing errors are silently ignored.*/
    pub fn get_all_attributes(&'a self) -> impl Iterator<Item = (String, String)> + 'a {
        self.element
            .attributes()
            .filter_map(|attr| {
                if attr.is_ok() {
                    Some(attr.unwrap())
                } else {
                    None
                }
            })
            .map(|attr| {
                (
                    qname_to_string(&attr.key),
                    String::from_utf8((*attr.value).to_vec()),
                )
            })
            .filter_map(|attr| {
                if attr.0.is_err() || attr.1.is_err() {
                    return None;
                }
                Some((attr.0.unwrap(), attr.1.unwrap()))
            })
    }

    /** Get a map of all attributes.

    If an attribute occurs multiple times, the last occurence is used.

    Parsing errors are silently ignored.*/
    pub fn get_attributes(&self) -> HashMap<String, String> {
        HashMap::from_iter(self.get_all_attributes())
    }

    /** Get an attribute. */
    pub fn get_attribute(&self, key: &str) -> Result<Option<String>, Error> {
        let Some(attr) = self.element.try_get_attribute(key)? else {
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

    /** Add or replace an attribute. */
    pub fn set_attribute(&mut self, key: &str, value: &str) -> Result<(), FromUtf8Error> {
        let mut attributes = self.get_attributes();
        attributes.insert(String::from(key), String::from(value));
        let attrs = attributes
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()));
        self.element.clear_attributes();
        self.element.extend_attributes(attrs);
        Ok(())
    }

    /** Change the tag name. */
    pub fn set_name(&mut self, name: &'a str) {
        self.element.set_name(name.as_bytes());
    }

    /** Get the tag name. */
    pub fn get_name(&self) -> Result<String, FromUtf8Error> {
        qname_to_string(&self.element.name())
    }
}

impl ToStringSafe for Element<'_> {
    fn to_string_safe(&self) -> Result<String, Error> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        for event in self.get_all_events() {
            let result = writer.write_event(event);
            if result.is_err() {
                return Err(result.unwrap_err());
            }
        }

        match String::from_utf8(writer.into_inner().into_inner()) {
            Ok(str) => Ok(str),
            Err(err) => Err(Error::NonDecodable(Some(err.utf8_error()))),
        }
    }
}

impl Display for Element<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self.to_string_safe().unwrap();
        write!(f, "{str}")
    }
}

impl GetEvents for Element<'_> {
    fn get_all_events(&self) -> Box<dyn Iterator<Item = Event> + '_> {
        if self.self_closing && self.children.is_empty() {
            Box::new(std::iter::once(Event::Empty(self.element.to_owned())))
        } else {
            let start_event = std::iter::once_with(|| Event::Start(self.element.to_owned()));
            let end_event = std::iter::once_with(|| Event::End(self.element.to_end()));

            let child_events = self
                .children
                .iter()
                .flat_map(|child| child.get_all_events());

            let events = start_event.chain(child_events.chain(end_event));

            Box::new(events)
        }
    }
}
