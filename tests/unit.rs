#[cfg(test)]
mod tests {
    use ilex_xml::*;
    use std::fs::read_to_string;

    #[test]
    fn test_echo() {
        let xml = read_to_string("test_data/small_inkscape.svg").unwrap();

        let items = parse(&xml).unwrap();

        let echo = items_to_string(&items);

        assert_eq!(xml, echo);
    }

    #[test]
    fn test_get_text_content() {
        let xml = read_to_string("test_data/tiny_people.xml").unwrap();

        let items = parse(&xml).unwrap();

        assert_eq!(items.len(), 1);

        let XmlItem::Element(element) = &items[0] else {
            panic!("Test data is corrupt.");
        };

        assert_eq!(element.get_text_content().unwrap(), "Bob99Alice123");
    }

    #[test]
    fn test_get_items_at_depth() {
        let xml = read_to_string("test_data/tiny_people.xml").unwrap();

        let items = parse(&xml).unwrap();

        assert_eq!(items.len(), 1);

        let XmlItem::Element(people_element) = &items[0] else {
            panic!("Test data is corrupt.");
        };
        let people = people_element.get_items_at_depth(1);

        assert_eq!(people.len(), 2);

        let mut people_info = Vec::new();

        for p in people {
            let XmlItem::Element(person) = p else {
                panic!("Item should be element.");
            };
            let datapoints = person.get_items_at_depth(1);
            assert_eq!(datapoints.len(), 2);
            let XmlItem::Element(name_element) = datapoints[0] else {
                panic!("Datapoint should be element.");
            };
            let name = name_element.get_text_content().unwrap();
            let XmlItem::Element(age_element) = datapoints[1] else {
                panic!("Datapoint should be element.");
            };
            let age = age_element.get_text_content().unwrap();
            people_info.push((name, age));
        }

        assert_eq!(people_info[0].0, "Bob");
        assert_eq!(people_info[0].1, "99");
        assert_eq!(people_info[1].0, "Alice");
        assert_eq!(people_info[1].1, "123");
    }

    #[test]
    fn test_get_attribute() {
        let xml = read_to_string("test_data/small_inkscape.svg").unwrap();

        let items = parse_trimmed(&xml).unwrap();

        let XmlItem::Element(item) = &items[2] else {
            panic!("Test data is corrupt.");
        };

        let value = item.get_attribute("id").unwrap().unwrap();

        assert_eq!(value, "svg1");

        let nothing = item.get_attribute("nonexistent-attribute").unwrap();

        assert!(nothing.is_none());
    }

    #[test]
    fn test_get_attributes() {
        let xml = read_to_string("test_data/small_inkscape.svg").unwrap();

        let items = parse_trimmed(&xml).unwrap();

        let XmlItem::Element(item) = &items[2] else {
            panic!("Test data is corrupt.");
        };

        let attrs = item.get_attributes().unwrap();

        assert_eq!(attrs.len(), 12);
        assert_eq!(attrs.get("id").unwrap(), "svg1");
    }

    #[test]
    fn test_get_attributes_empty() {
        let xml = "<a></a>";

        let item = &parse(xml).unwrap()[0];

        let XmlItem::Element(element) = item else {
            panic!("Test data is corrupt.");
        };

        let attrs = element.get_attributes().unwrap();

        assert_eq!(attrs.len(), 0);
    }

    #[test]
    fn test_add_attribute() {
        let xml = r#"<x></x><a></a><y></y>"#;

        let mut items = parse(&xml).unwrap();

        let XmlItem::Element(element) = &mut items[1] else {
            panic!("Test data is corrupt.");
        };

        element.set_attribute("works", "yes").unwrap();

        let modified_xml = items_to_string(&items);

        assert_eq!(modified_xml, r#"<x></x><a works="yes"></a><y></y>"#);
    }

    #[test]
    fn test_replace_attribute() {
        let xml = r#"<x></x><a works="no"></a><y></y>"#;

        let mut items = parse(&xml).unwrap();

        let XmlItem::Element(element) = &mut items[1] else {
            panic!("Test data is corrupt.");
        };

        element.set_attribute("works", "yes").unwrap();

        let modified_xml = items_to_string(&items);

        assert_eq!(modified_xml, r#"<x></x><a works="yes"></a><y></y>"#);
    }

    #[test]
    fn test_add_children() {
        let xml = "<a></a><b><c></c></b>";

        let mut items = parse(&xml).unwrap();

        items.push(XmlItem::Element(Element::new("x")));

        let XmlItem::Element(element_a) = &mut items[0] else {
            panic!("Test data is corrupt.");
        };

        element_a
            .children
            .push(XmlItem::Text(Other::new_text("works")));

        let XmlItem::Element(element_b) = &mut items[1] else {
            panic!("Test data is corrupt.");
        };

        element_b
            .children
            .push(XmlItem::EmptyElement(EmptyElement::new("z")));

        let modified_xml = items_to_string(&items);

        assert_eq!(modified_xml, "<a>works</a><b><c></c><z/></b><x></x>");
    }

    #[test]
    fn test_get_name() {
        let xml = "<a></a>";

        let item = &parse(&xml).unwrap()[0];

        let XmlItem::Element(element) = item else {
            panic!("Test data is corrupt.");
        };

        assert_eq!(element.get_name().unwrap(), "a");
    }

    #[test]
    fn test_set_name() {
        let xml = "<test></test>";

        let mut items = parse(&xml).unwrap();

        let XmlItem::Element(element) = &mut items[0] else {
            panic!("Test data is corrupt.");
        };

        element.set_name("works");

        assert_eq!(element.to_string(), "<works></works>");
    }

    #[test]
    fn test_get_value() {
        let xml = "hey";

        let items = parse(&xml).unwrap();

        let XmlItem::Text(text) = &items[0] else {
            panic!("Test data is corrupt.");
        };

        assert_eq!(text.get_value().unwrap(), "hey");
    }

    #[test]
    fn test_set_value() {
        let xml = "test";

        let mut items = parse(&xml).unwrap();

        let XmlItem::Text(text) = &mut items[0] else {
            panic!("Test data is corrupt.");
        };

        text.set_value("works");

        assert_eq!(items_to_string(&items), "works");
    }
}
