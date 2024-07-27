# ilex
Simple tree structure XML library.

## Example
```Rust
use ilex_xml::{items_to_string, parse_trimmed, Other, Tag, XmlItem};

let xml = r#"
    <!-- The cat is cute. -->
    <parent>
        <child likes="orange">Alice</child>
        <child likes="teal">Bob</child>
    </parent>
"#;

let mut items = parse_trimmed(xml).unwrap();

{ // Get comment content
    let XmlItem::Comment(comment) = &items[0] else {
        panic!(
            "Huh, odd. Let's look at the first element's raw XML: {}",
            items[0]
        );
    };

    println!("I found a useful comment:{}", comment.get_value().unwrap());
}

let XmlItem::Element(parent) = &mut items[1] else {
    panic!("Pretty sure the second item is an element.")
};

{ // Print attributes and text contents of children
    for item in &parent.children {
        let XmlItem::Element(child) = item else {
            panic!("The children are elements, too.")
        };

        let name = child.get_text_content().unwrap();
        let color = child.get_attribute("likes").unwrap().unwrap();

        println!("{name}'s favorite color is {color}!");
    }
}

println!("Hey, their name isn't Bob! It's Peter!");

{ // Replace child
    let XmlItem::Element(child) = &mut parent.children[1] else {
        panic!();
    };

    // Remove the wrong name
    child.children.remove(0);
    // Add the correct name
    child.children.push(XmlItem::Text(Other::new_text("Peter")));

    println!(
        "Lets take another look at the raw XML, now that the name is fixed: {}",
        items_to_string(&items)
    );
}
```
