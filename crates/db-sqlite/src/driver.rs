use std::path::Path;

use db_api::{DatabaseError, result::ToDatabaseError};
use sqlx::{
    Sqlite, SqliteTransaction,
    pool::PoolConnection,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

/// SQLite database migrator
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// An opened SQLite database
///
/// Provides access to the database through a connection pool
#[derive(Debug, Clone)]
pub struct SqliteDatabase {
    /// Database connection pool
    pool: sqlx::SqlitePool,
}

impl SqliteDatabase {
    /// Opens an [`SqliteDatabase`] from a file at the specified path
    ///
    /// Creates the database file and its parent directories if they do not exist
    pub async fn open(p: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        let path = p.as_ref();

        if !path.exists() {
            tracing::warn!("SQLite database file does not exist. Creating a new one!");
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).to_db_error()?;
        }

        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .auto_vacuum(SqliteAutoVacuum::Incremental)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .connect_with(opts)
            .await
            .to_db_error()?;

        tracing::info!(file_path = ?path, "SQLite database opened");

        Ok(SqliteDatabase { pool })
    }

    /// Applies migrations to an open database
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        MIGRATOR.run(&self.pool).await.to_db_error()
    }

    /// Returns a reference to the database connection pool
    pub(crate) fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }

    /// Acquires a connection from the pool and returns it
    ///
    /// The connection is returned to the pool when dropped
    pub(crate) async fn acquire(&self) -> Result<PoolConnection<Sqlite>, DatabaseError> {
        self.pool.acquire().await.to_db_error()
    }

    /// Begins a database transaction
    pub(crate) async fn begin(&self) -> Result<SqliteTransaction<'_>, DatabaseError> {
        self.pool.begin().await.to_db_error()
    }
}
