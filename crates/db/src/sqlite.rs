use std::path::Path;

use result::{Result, error::ResultExt};
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use crate::database::{Database, DatabaseProvider, DatabaseSession, DatabaseTransaction};

static SQLITE_MIGRATOR: Migrator = sqlx::migrate!();

/// Abstract executor for [`SqliteDatabase`]
pub(crate) trait SqliteExecutor {
    /// Returns the executor for the current session.
    fn executor(&mut self) -> &mut sqlx::SqliteConnection;
}

/// Opened SQLite database
#[derive(Clone)]
pub struct SqliteDatabase {
    /// Database connections pool
    pool: SqlitePool,
}

/// Active [`SqliteDatabase`] session
pub struct SqliteSession(sqlx::pool::PoolConnection<sqlx::Sqlite>);

/// Opened [`SqliteDatabase`] transaction
pub struct SqliteTransaction(sqlx::SqliteTransaction<'static>);

impl SqliteDatabase {
    /// Opens a [`SqliteDatabase`] from a file
    pub async fn open(p: impl AsRef<Path>) -> Result<Self> {
        let path = p.as_ref();

        if !path.exists() {
            tracing::info!("Database file not found. Creating a new one!");
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).to_app_err()?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .auto_vacuum(SqliteAutoVacuum::Incremental)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .to_app_err()?;
        Ok(SqliteDatabase { pool })
    }

    /// Applies migrations to the [`SqliteDatabase`]
    pub async fn migrate(&self) -> Result<()> {
        SQLITE_MIGRATOR.run(&self.pool).await.to_app_err()
    }
}

impl Database for SqliteDatabase {
    type Session = SqliteSession;
    type Transaction = SqliteTransaction;
}

impl DatabaseProvider for SqliteDatabase {
    async fn acquire(&self) -> Result<Self::Session> {
        let conn = self.pool.acquire().await.to_app_err()?;
        Ok(SqliteSession(conn))
    }

    async fn begin(&self) -> Result<Self::Transaction> {
        let tx = self.pool.begin().await.to_app_err()?;
        Ok(SqliteTransaction(tx))
    }
}

impl DatabaseTransaction for SqliteTransaction {
    async fn commit(self) -> Result<()> {
        self.0.commit().await.to_app_err()
    }

    async fn rollback(self) -> Result<()> {
        self.0.rollback().await.to_app_err()
    }
}

impl<T: SqliteExecutor> DatabaseSession for T {}

impl SqliteExecutor for SqliteTransaction {
    fn executor(&mut self) -> &mut sqlx::SqliteConnection {
        &mut self.0
    }
}

impl SqliteExecutor for SqliteSession {
    fn executor(&mut self) -> &mut sqlx::SqliteConnection {
        &mut self.0
    }
}
