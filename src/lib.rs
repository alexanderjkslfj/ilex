//! Tree structure XML reader/writer.
//! Focus on ease of use while retaining good performance.
//!
//! This library is a wrapper for [quick_xml].
//!
//! [quick_xml]: https://docs.rs/quick-xml/latest/quick_xml/

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod element;
mod item;
mod other;
mod parsing;
mod traits;
mod util;

pub use element::*;
pub use item::*;
pub use other::*;
pub use parsing::*;
pub use quick_xml::Error;
