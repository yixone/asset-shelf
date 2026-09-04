use std::sync::Arc;

pub use db_core::repos::RepositoryContext;

pub use db_core::{ops, repos, types};

use db_sqlite::SqliteDatabase;

/// Selected database driver
pub enum Database {
    /// Sqlite DB driver
    Sqlite(SqliteDatabase),
}

impl Database {
    /// Opens an [`SqliteDatabase`] from the specified file and applies migrations
    pub async fn sqlite(file: impl AsRef<std::path::Path>) -> result::Result<Self> {
        let db = SqliteDatabase::open(file).await?;
        db.migrate().await?;

        Ok(Database::Sqlite(db))
    }

    /// Creates a [`RepositoryContext`] using the current [`Database`]
    pub fn repos(self) -> RepositoryContext {
        match self {
            Database::Sqlite(db) => db.repositories(),
        }
    }

    /// Creates a [`RepositoryContext`] using the current [`Database`]
    /// and wraps this context in an [`Arc`] for sharing between threads
    pub fn repos_shared(self) -> Arc<RepositoryContext> {
        Arc::new(self.repos())
    }
}
