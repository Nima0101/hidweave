//! Offline comparison of bounded HID 1.11 report contracts.
//!
//! No device access, host timezone/locale, environment configuration, or network is used.
//! Unsupported descriptor semantics return an error; they never compare as unchanged.
//!
//! ```
//! let bytes = hidweave::parse_hex(include_str!("../examples/axis-old.hex")).unwrap();
//! let layout = hidweave::parse(&bytes).unwrap();
//! assert!(hidweave::compare(&layout, &layout).is_empty());
//! ```
mod compare;
mod decode;
pub mod gate;
pub mod json;
mod model;
mod parser;

pub use compare::{Change, ChangeKind, compare};
pub use decode::{DecodedValue, ValueState, decode};
pub use model::*;
pub use parser::{parse, parse_hex};
