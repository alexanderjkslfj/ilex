#[cfg(test)]
mod tests {
    use ilex_xml::*;
    use std::{fs::read_to_string, num::NonZero};

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

        let Item::Element(element) = &items[0] else {
            panic!("Test data is corrupt.");
        };

        assert_eq!(element.get_text_content(), "Bob99Alice123");
    }

    #[test]
    fn test_get_items_at_depth() {
        let xml = read_to_string("test_data/tiny_people.xml").unwrap();

        let items = parse(&xml).unwrap();

        assert_eq!(items.len(), 1);

        let Item::Element(people_element) = &items[0] else {
            panic!("Test data is corrupt.");
        };
        let people: Vec<_> = people_element
            .get_items_at_depth(NonZero::new(1).unwrap())
            .collect();

        assert_eq!(people.len(), 2);

        let mut people_info = Vec::new();

        for p in people {
            let Item::Element(person) = p else {
                panic!("Item should be element.");
            };
            let datapoints: Vec<_> = person
                .get_items_at_depth(NonZero::new(1).unwrap())
                .collect();
            assert_eq!(datapoints.len(), 2);
            let Item::Element(name_element) = datapoints[0] else {
                panic!("Datapoint should be element.");
            };
            let name = name_element.get_text_content();
            let Item::Element(age_element) = datapoints[1] else {
                panic!("Datapoint should be element.");
            };
            let age = age_element.get_text_content();
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

        let Item::Element(item) = &items[2] else {
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

        let Item::Element(item) = &items[2] else {
            panic!("Test data is corrupt.");
        };

        let attrs = item.get_attributes();

        assert_eq!(attrs.len(), 12);
        assert_eq!(attrs.get("id").unwrap(), "svg1");
    }

    #[test]
    fn test_get_attributes_empty() {
        let xml = "<a></a>";

        let item = &parse(xml).unwrap()[0];

        let Item::Element(element) = item else {
            panic!("Test data is corrupt.");
        };

        let attrs = element.get_attributes();

        assert_eq!(attrs.len(), 0);
    }

    #[test]
    fn test_add_attribute() {
        let xml = r#"<x></x><a></a><y></y>"#;

        let mut items = parse(&xml).unwrap();

        let Item::Element(element) = &mut items[1] else {
            panic!("Test data is corrupt.");
        };

        element.set_attribute("works", "yes");

        let modified_xml = items_to_string(&items);

        assert_eq!(modified_xml, r#"<x></x><a works="yes"></a><y></y>"#);
    }

    #[test]
    fn test_replace_attribute() {
        let xml = r#"<x></x><a works="no"></a><y></y>"#;

        let mut items = parse(&xml).unwrap();

        let Item::Element(element) = &mut items[1] else {
            panic!("Test data is corrupt.");
        };

        element.set_attribute("works", "yes");

        let modified_xml = items_to_string(&items);

        assert_eq!(modified_xml, r#"<x></x><a works="yes"></a><y></y>"#);
    }

    #[test]
    fn test_add_children() {
        let xml = "<a></a><b><c></c></b>";

        let mut items = parse(&xml).unwrap();

        items.push(Item::new_element("x", false));

        let Item::Element(element_a) = &mut items[0] else {
            panic!("Test data is corrupt.");
        };

        element_a
            .children
            .push(Item::Text(Other::new_text("works")));

        let Item::Element(element_b) = &mut items[1] else {
            panic!("Test data is corrupt.");
        };

        element_b.children.push(Item::new_element("z", true));

        let modified_xml = items_to_string(&items);

        assert_eq!(modified_xml, "<a>works</a><b><c></c><z/></b><x></x>");
    }

    #[test]
    fn test_get_name() {
        let xml = "<a></a>";

        let item = &parse(&xml).unwrap()[0];

        let Item::Element(element) = item else {
            panic!("Test data is corrupt.");
        };

        assert_eq!(element.get_name().unwrap(), "a");
    }

    #[test]
    fn test_set_name() {
        let xml = "<test></test>";

        let mut items = parse(&xml).unwrap();

        let Item::Element(element) = &mut items[0] else {
            panic!("Test data is corrupt.");
        };

        element.set_name("works");

        assert_eq!(element.to_string(), "<works></works>");
    }

    #[test]
    fn test_get_value() {
        let xml = "hey";

        let items = parse(&xml).unwrap();

        let Item::Text(text) = &items[0] else {
            panic!("Test data is corrupt.");
        };

        assert_eq!(text.get_value().unwrap(), "hey");
    }

    #[test]
    fn test_find_descendants() {
        let xml =
            r#"<a key="1"><b key="1"/><c key="0"/><d key="0"><e key="1">Some Text</e></d></a>"#;

        let items = parse(&xml).unwrap();

        let Item::Element(a) = &items[0] else {
            panic!("Test data is corrupt.");
        };

        let desc_it = a.find_descendants(&|item| match item {
            Item::Element(el) => el.get_attribute("key").unwrap().unwrap() == "1",
            _ => false,
        });

        let descs: Vec<_> = desc_it.collect();

        assert_eq!(descs.len(), 2);
        assert_eq!(descs[0].to_string(), r#"<b key="1"/>"#);
        assert_eq!(descs[1].to_string(), r#"<e key="1">Some Text</e>"#);
    }
}
