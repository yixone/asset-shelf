//! tsid — a identifier based on a timestamp

#![allow(clippy::new_without_default)]

pub mod generator;
pub mod id;
pub mod str;

pub use generator::FlakeIdGenerator;
pub use id::FlakeId;
