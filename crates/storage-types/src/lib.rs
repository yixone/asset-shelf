//! Value types used by the managed file storage subsystem
//!
//! This crates defines implementation-independent types used to
//! describe files, paths, and storage statistics
//!
//! It contains no filesystem operations or storage implementations

pub mod meta;
pub mod path;
pub mod stats;

pub use meta::FileMetadata;
pub use path::StoragePath;
pub use stats::DiskUsageStats;
