//! Tree structure XML reader/writer.
//! Focus on ease of use while maintaining good performance.
//!
//! ```rust
//! # use ilex_xml::*;
//! let xml = r#"<color hue="100" brightness="50" />"#;
//!
//! let item = &parse(xml)?[0];
//!
//! let Item::Element(element) = item else {
//!     panic!();
//! };
//!
//! let attrs = element.get_attributes();
//!
//! assert_eq!(attrs.get("hue").unwrap(), "100");
//! assert_eq!(attrs.get("brightness").unwrap(), "50");
//! # Ok::<(), Error>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod element;
mod item;
mod other;
mod parsing;
mod util;

pub use element::*;
pub use item::*;
pub use other::*;
pub use parsing::*;
pub use quick_xml::Error;
pub use util::ToStringSafe;
