use std::{collections::HashMap, string::FromUtf8Error};

use quick_xml::events::Event;

use crate::Error;

pub trait GetEvents {
    fn get_all_events(&self) -> Box<dyn Iterator<Item = Event> + '_>;
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
