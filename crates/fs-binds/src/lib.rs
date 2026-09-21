//! Low-level bindigs to platform-specific FS APIs
//!
//! The crate isolates `unsafe` and OS-specific code
//! behind small safe interfaces. Implementations are selected
//! according to the target platform

#[cfg(not(any(unix, windows)))]
compile_error!("Unsupported OS for fs-bindings");

mod rename;
pub mod stats;

pub use stats::statvfs;
