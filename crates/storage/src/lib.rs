//! Managed file storage for AssetShelf
//!
//! The storage manages files under a single physical root directory.
//! Files are addressed using logical [`path::StoragePath`] values
//! rather than physical fs paths
//!
//! New files are written to temporary staging paths and published
//! using exclusive rename operations. This prevents incomplete files
//! from being visible at their final storage paths
//!
//! The storage provides file readers, writers, staged file management,
//! filesystem stats, and basic file operations

// The file storage works only on Linux, MacOS, and Windows
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
compile_error!("Unsupported operating system");

pub mod file;
pub mod layout;

pub(crate) mod fs;
pub(crate) mod utils;

pub mod result;

pub mod mount;
pub mod storage;

pub use storage::Storage;
