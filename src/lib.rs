//! Tree structure XML reader/writer.
//! Focus on ease of use while maintaining good performance.

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
