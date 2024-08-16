use std::{collections::HashMap, string::FromUtf8Error};

use crate::Error;

use quick_xml::{events::{attributes::Attribute, BytesStart}, name::QName};

pub fn qname_to_string(qname: &QName) -> Result<String, FromUtf8Error> {
    u8_to_string(qname.as_ref())
}

pub fn u8_to_string(u8: &[u8]) -> Result<String, FromUtf8Error> {
    String::from_utf8(u8.to_vec())
}

pub fn get_attributes(start: &BytesStart) -> Result<HashMap<String, String>, FromUtf8Error> {
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
        let value = String::from_utf8((*attr.value).to_vec())?;
        attributes.insert(key, value);
    }

    Ok(attributes)
}

pub fn get_attribute(start: &BytesStart, key: &str) -> Result<Option<String>, Error> {
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

pub fn set_attribute(start: &mut BytesStart, key: &str, value: &str) -> Result<(), FromUtf8Error> {
    let mut attributes = get_attributes(start)?;
    attributes.insert(String::from(key), String::from(value));
    let attrs = attributes
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()));
    start.clear_attributes();
    start.extend_attributes(attrs);
    Ok(())
}