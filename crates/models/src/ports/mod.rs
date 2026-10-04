//! Persistence contracts used by the domain
//!
//! The traits in this module describe operations required by domain models
//! to persist state. They are independent of the underlying storage
//! implementation and must not expose database-specific models or types
//!
//! Infrastructure crates provide concrete implementations of these contracts

pub mod asset;
pub mod file;

pub use asset::*;
pub use file::*;

use crate::result::Result;

/// Represents a unit of work whose changes can be commited or discarded
///
/// A unit of work groups one or more persistence operations into a single
/// atomic operation. Implementations must ensure that changes are either
/// commited together or discarded when the unit of work is rolled back
#[async_trait::async_trait]
pub trait UnitOfWork {
    /// Commits all changes made within this unit of work
    ///
    /// After a successful commit, the changes becomes permanent and the
    /// unit of work can no longer be used
    async fn commit(self) -> Result<()>;

    /// Rolls back all changes made within this unit of work
    ///
    /// After a successful rollback, none of the changes made within this
    /// unit of work are persisted and the unit of work can no longer be used
    async fn rollback(self) -> Result<()>;
}
