//! SQLite persistence schema
//!
//! This module defines database-facing models used to read and write
//! application data in SQLite

// TODO: Rebuild `Domain` <-> `Db` mapping

pub mod asset;
pub mod file;

pub use asset::{DbAsset, DbAssetFeatures, DbAssetMeta};
pub use file::{DbFile, DbFileGroup};
