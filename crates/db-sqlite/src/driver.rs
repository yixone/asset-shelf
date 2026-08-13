use std::{path::Path, sync::Arc};

use db_core::repos::RepositoryContext;
use result::{Result, error::ResultExt};
use sqlx::{
    Sqlite, SqlitePool, SqliteTransaction,
    migrate::Migrator,
    pool::PoolConnection,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use crate::repos::{
    asset::SqliteAssetRepository, collection::SqliteCollectionRepository,
    media::SqliteMediaRepository,
};

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

    pub(crate) fn exec(&self) -> &SqlitePool {
        &self.pool
    }

    pub(crate) async fn acquire(&self) -> Result<PoolConnection<Sqlite>> {
        self.pool.acquire().await.to_app_err()
    }

    pub(crate) async fn begin(&self) -> Result<SqliteTransaction<'_>> {
        self.pool.begin().await.to_app_err()
    }

    pub fn repositories(self) -> RepositoryContext {
        let db = Arc::new(self);
        RepositoryContext {
            assets: Arc::new(SqliteAssetRepository { db: db.clone() }),
            collections: Arc::new(SqliteCollectionRepository { db: db.clone() }),
            media: Arc::new(SqliteMediaRepository { db: db.clone() }),
        }
    }
}
