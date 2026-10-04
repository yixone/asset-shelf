//! Persistence contracts used by the domain
//!
//! The traits in this module describe operations required by domain models
//! to persist state. They are independent of the underlying storage
//! implementation and must not expose database-specific models or types
//!
//! Infrastructure crates provide concrete implementations of these contracts

pub mod asset;
pub mod file;

pub use file::*;
