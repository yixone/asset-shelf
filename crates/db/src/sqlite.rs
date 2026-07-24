use std::path::Path;

use result::{Result, error::ResultExt};
use sqlx::{
    Sqlite, SqliteConnection, SqlitePool, SqliteTransaction,
    migrate::Migrator,
    pool::PoolConnection,
    sqlite::{
        SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions,
        SqliteQueryResult,
    },
};

use crate::core::{
    provider::{ConnectionUnit, DatabaseConnector, DatabaseProvider, TransactionUnit},
    result::{DeleteResult, InsertResult, UpdateResult},
};

static SQLITE_MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Clone)]
pub struct SqliteDb {
    pool: SqlitePool,
}

impl SqliteDb {
    pub async fn open(p: impl AsRef<Path>) -> Result<Self> {
        let path = p.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).to_app_err()?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .auto_vacuum(SqliteAutoVacuum::Incremental)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .to_app_err()?;
        Ok(SqliteDb { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        SQLITE_MIGRATOR.run(&self.pool).await.to_app_err()?;
        Ok(())
    }
}

pub struct SqliteTx<'a> {
    tx: SqliteTransaction<'a>,
}

pub struct SqliteConn {
    conn: PoolConnection<Sqlite>,
}

impl DatabaseProvider for SqliteDb {
    type Connection = SqliteConn;
    type Transaction<'a> = SqliteTx<'a>;
}

impl DatabaseConnector for SqliteDb {
    async fn acquire(&self) -> Result<Self::Connection> {
        let conn = self.pool.acquire().await.to_app_err()?;
        Ok(SqliteConn { conn })
    }
    async fn begin(&self) -> Result<Self::Transaction<'_>> {
        let tx = self.pool.begin().await.to_app_err()?;
        Ok(SqliteTx { tx })
    }
}

pub(crate) trait SqliteUnit {
    fn exec(&mut self) -> &mut SqliteConnection;
}

impl SqliteUnit for SqliteTx<'_> {
    fn exec(&mut self) -> &mut SqliteConnection {
        &mut self.tx
    }
}

impl SqliteUnit for SqliteConn {
    fn exec(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }
}

impl ConnectionUnit for SqliteConn {
    type Error = sqlx::Error;
}

impl TransactionUnit for SqliteTx<'_> {
    async fn commit(self) -> Result<()> {
        self.tx.commit().await.to_app_err()
    }
    async fn rollback(self) -> Result<()> {
        self.tx.rollback().await.to_app_err()
    }
}

impl From<SqliteQueryResult> for InsertResult {
    fn from(res: SqliteQueryResult) -> Self {
        if res.rows_affected() == 0 {
            InsertResult::NoChanges
        } else {
            InsertResult::Inserted
        }
    }
}
impl From<SqliteQueryResult> for UpdateResult {
    fn from(res: SqliteQueryResult) -> Self {
        if res.rows_affected() == 0 {
            UpdateResult::NoChanges
        } else {
            UpdateResult::Updated(res.rows_affected())
        }
    }
}
impl From<SqliteQueryResult> for DeleteResult {
    fn from(res: SqliteQueryResult) -> Self {
        if res.rows_affected() == 0 {
            DeleteResult::NoChanges
        } else {
            DeleteResult::Deleted(res.rows_affected())
        }
    }
}
