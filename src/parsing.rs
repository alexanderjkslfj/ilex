use crate::{util::qname_to_string, Element, Error, Item, Other};
use quick_xml::{
    errors::IllFormedError,
    events::Event,
    Reader,
};

/** Parse raw XML and trim whitespace at the front and end of text. */
pub fn parse_trimmed(xml: &str) -> Result<Vec<Item>, Error> {
    let events = get_all_events(xml, true)?;
    Ok(parse_events(&events)?)
}

/** Parse raw XML. */
pub fn parse(xml: &str) -> Result<Vec<Item>, Error> {
    let events = get_all_events(xml, false)?;
    Ok(parse_events(&events)?)
}

fn parse_events<'a>(events: &[Event<'a>]) -> Result<Vec<Item<'a>>, Error> {
    let mut items = Vec::new();

    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Text(item) => items.push(Item::Text(Other::Text(item.to_owned()))),
            Event::Comment(item) => items.push(Item::Comment(Other::Comment(item.to_owned()))),
            Event::CData(item) => items.push(Item::CData(Other::CData(item.to_owned()))),
            Event::PI(item) => items.push(Item::PI(Other::PI(item.to_owned()))),
            Event::Decl(item) => items.push(Item::Decl(Other::Decl(item.to_owned()))),
            Event::DocType(item) => items.push(Item::DocType(Other::DocType(item.to_owned()))),
            Event::Empty(item) => items.push(Item::Element(Element {
                element: item.to_owned(),
                children: Vec::new(),
                self_closing: true,
            })),
            Event::Start(start) => {
                let mut depth = 1;
                let mut sub_events = Vec::new();
                loop {
                    i += 1;
                    let event = &events[i];
                    match event {
                        Event::Start(_) => {
                            depth += 1;
                        }
                        Event::End(_) => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => (),
                    }
                    sub_events.push(event.to_owned());
                }
                items.push(Item::Element(Element {
                    element: start.to_owned(),
                    children: parse_events(&sub_events)?,
                    self_closing: false,
                }));
            }
            Event::End(end) => {
                let name = qname_to_string(&end.name());
                if name.is_ok() {
                    return Err(Error::IllFormed(IllFormedError::UnmatchedEndTag(
                        name.unwrap(),
                    )));
                } else {
                    return Err(Error::NonDecodable(Some(name.unwrap_err().utf8_error())));
                };
            }
            Event::Eof => {
                panic!("An unexpected internal error occured in the ilex_xml library. This should never happen. Please issue a report to the maintainers.");
            },
        }
        i += 1;
    }

    Ok(items)
}

fn get_all_events(xml: &str, trim: bool) -> Result<Vec<Event>, Error> {
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

/** Stringify a list of XML items.

Equivalent to calling ```to_string``` on each item and concatenating the results.
*/
pub fn items_to_string(items: &[Item]) -> String {
    let mut str = String::new();
    for item in items {
        let item_str = match &item {
            Item::Text(text) => text.to_string(),
            Item::Comment(text) => text.to_string(),
            Item::CData(text) => text.to_string(),
            Item::PI(text) => text.to_string(),
            Item::Decl(text) => text.to_string(),
            Item::DocType(text) => text.to_string(),
            Item::Element(text) => text.to_string(),
        };
        str.push_str(&item_str);
    }
    str
}
