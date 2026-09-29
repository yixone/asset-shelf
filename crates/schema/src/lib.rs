//! Domain types and structures used in the application
//!
//! Types defined in this crate represent the application's core concepts
//! and are primarily used used as internal types
//!
//! ## Integration with the Rust crates ecosystem:
//!
//! Support for third-party crates is optional and enabled via dependency features:
//!
//! - `sqlx`: Enables [`sqlx::Type`] and [`sqlx::FromRow`]
//!   implementations for types defined in this crate
//! - `serde`: Enables [`serde::Serialize`] and [`serde::Deserialize`]
//!   implementations for supported types, excluding domain entities

pub mod event;
pub mod id;

pub mod entity;
pub mod types;
