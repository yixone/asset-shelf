//! Low-level cross-platform file system APIs
//!
//! The crate provides safe Rust interfaces over platform-specific
//! operating system file system APIs

#[cfg(not(any(unix, windows)))]
compile_error!("Unsupported operating system");

mod rename;
pub mod stats;

pub use stats::statvfs;
