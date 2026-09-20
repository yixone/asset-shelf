//! Types and utilites for media type identification
//!
//! Module provides MIME types representations and detectors for
//! identifying media types from file magic bytes

#[macro_use]
mod pattern;

mod kind;
mod mime;

mod guess;

pub use guess::guess_mime;
pub use kind::MimeKind;
pub use mime::MimeType;
