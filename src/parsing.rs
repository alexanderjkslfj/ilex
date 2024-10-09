use crate::{util::qname_to_string, Element, Error, Item, Other, ToStringSafe};
use quick_xml::{errors::IllFormedError, events::Event, Reader};

/** Parse raw XML and trim whitespace at the front and end of text. */
pub fn parse_trimmed(xml: &str) -> Result<Vec<Item>, Error> {
    let events = read_events(xml, true);
    Ok(parse_events(events)?)
}

/** Parse raw XML. */
pub fn parse(xml: &str) -> Result<Vec<Item>, Error> {
    let events = read_events(xml, false);
    Ok(parse_events(events)?)
}

fn parse_events<'a>(mut events: impl Iterator<Item = Result<Event<'a>, Error>>) -> Result<Vec<Item<'a>>, Error> {
    let mut items = Vec::new();

    while let Some(next) = events.next() {
        match next? {
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
                    let Some(Ok(event)) = events.next() else {
                        let name = qname_to_string(&start.name());
                        return Err(Error::IllFormed(IllFormedError::MissingEndTag(
                            name.unwrap_or(String::new()),
                        )));
                    };
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
                    sub_events.push(Ok(event.to_owned()));
                }
                items.push(Item::Element(Element {
                    element: start.to_owned(),
                    children: parse_events(sub_events.into_iter())?,
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
                unreachable!();
            }
        }
    }

    return Ok(items);
}

struct EventIterator<'a> {
    reader: Reader<&'a [u8]>
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Result<Event<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_event() {
            Err(err) => Some(Err(err)),

            Ok(Event::Eof) => None,

            Ok(e) => Some(Ok(e)),
        }
    }
}

fn read_events(xml: &str, trim: bool) -> impl Iterator<Item = Result<Event, Error>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(trim);
    EventIterator { reader }
}

/** Stringify a list of XML items.

Equivalent to calling `to_string` on each item and concatenating the results.

Parsing errors are silently ignored.*/
pub fn items_to_string(items: &[Item]) -> String {
    items
        .iter()
        .map(|item| item.to_string_safe())
        .filter_map(|result| match result {
            Ok(str) => Some(str),
            Err(_) => None,
        })
        .collect()
}
