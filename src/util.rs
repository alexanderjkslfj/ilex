use quick_xml::events::Event;
use quick_xml::name::QName;
use std::string::FromUtf8Error;

pub fn qname_to_string(qname: &QName) -> Result<String, FromUtf8Error> {
    u8_to_string(qname.as_ref())
}

pub fn u8_to_string(u8: &[u8]) -> Result<String, FromUtf8Error> {
    String::from_utf8(u8.to_vec())
}

pub trait GetEvents {
    fn get_all_events(&self) -> Box<dyn Iterator<Item = Event> + '_>;
}

/** A trait for converting an item to a String without panicing. */
pub trait ToStringSafe {
    /** Converts the given item to a `String`, but doesn't panic. */
    fn to_string_safe(&self) -> Result<String, crate::Error>;
}
