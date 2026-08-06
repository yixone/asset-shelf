use std::path::Path;

use result::{Result, error::ResultExt};
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use crate::database::{Database, DatabaseProvider, DatabaseTransaction};

static SQLITE_MIGRATOR: Migrator = sqlx::migrate!();

/// Opened SQLite database
#[derive(Clone)]
pub struct SqliteDatabase {
    /// Database connections pool
    pool: SqlitePool,
}

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

    /// Opens a temporary [`SqliteDatabase`] in memory
    pub async fn open_in_mem() -> Result<Self> {
        let options = SqliteConnectOptions::new().in_memory(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
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

/// Active [`SqliteDatabase`] session
pub struct SqliteSession {
    pub(crate) conn: sqlx::pool::PoolConnection<sqlx::Sqlite>,
}

/// Opened [`SqliteDatabase`] transaction
pub struct SqliteTransaction {
    pub(crate) tx: sqlx::SqliteTransaction<'static>,
}

impl Database for SqliteDatabase {
    type Session = SqliteSession;
    type Transaction = SqliteTransaction;
}

impl DatabaseProvider for SqliteDatabase {
    async fn acquire(&self) -> Result<Self::Session> {
        let conn = self.pool.acquire().await.to_app_err()?;
        Ok(SqliteSession { conn })
    }

    async fn begin(&self) -> Result<Self::Transaction> {
        let tx = self.pool.begin().await.to_app_err()?;
        Ok(SqliteTransaction { tx })
    }
}

impl DatabaseTransaction for SqliteTransaction {
    async fn commit(self) -> Result<()> {
        self.tx.commit().await.to_app_err()
    }

    async fn rollback(self) -> Result<()> {
        self.tx.rollback().await.to_app_err()
    }
}
