// The file storage works only on Linux, MacOS, and Windows
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
compile_error!("Unsupported operating system");

pub mod file;
pub mod layout;
pub mod path;

pub mod fs;
pub mod types;
pub mod utils;

pub mod result;

pub mod mount;
pub mod storage;
