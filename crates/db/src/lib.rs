use std::path::Path;
use std::sync::Arc;

pub use db_core::repos::RepositoryContext;

pub use db_core::{ops, repos, types};

use db_sqlite::SqliteDatabase;
use result::Result;

/// Selected database driver
pub enum Database {
    /// Sqlite DB driver
    Sqlite(Arc<SqliteDatabase>),
}

impl Database {
    /// Opens an [`SqliteDatabase`] from the specified file and applies migrations
    pub async fn sqlite(file: impl AsRef<std::path::Path>) -> result::Result<Self> {
        let db = SqliteDatabase::open(file).await?;
        db.migrate().await?;

        let shared = Arc::new(db);

        Ok(Database::Sqlite(shared))
    }

    /// Creates a [`RepositoryContext`] using the current [`Database`]
    pub fn repos(&self) -> RepositoryContext {
        match self {
            Database::Sqlite(db) => db.repositories(),
        }
    }

    /// Creates a [`RepositoryContext`] using the current [`Database`]
    /// and wraps this context in an [`Arc`] for sharing between threads
    pub fn repos_shared(&self) -> Arc<RepositoryContext> {
        Arc::new(self.repos())
    }

    /// Backs up the [`Database`] to the specified file
    pub async fn backup(&self, path: &Path) -> Result<()> {
        match self {
            Database::Sqlite(db) => db.backup(&path.display().to_string()).await,
        }
    }
}
