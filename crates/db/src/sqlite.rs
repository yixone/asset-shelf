use sqlx::{
    Sqlite, SqliteConnection, SqlitePool, SqliteTransaction, pool::PoolConnection,
    sqlite::SqliteQueryResult,
};

use crate::{
    core::result::{DeleteResult, InsertResult, UpdateResult},
    provider::{ConnectionUnit, DatabaseProvider, DatabaseProviderExt, TransactionUnit},
};

pub struct SqliteDb {
    pool: SqlitePool,
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

impl DatabaseProviderExt for SqliteDb {
    type Error = sqlx::Error;

    async fn acquire(&self) -> Result<Self::Connection, Self::Error> {
        let conn = self.pool.acquire().await?;
        Ok(SqliteConn { conn })
    }
    async fn begin(&self) -> Result<Self::Transaction<'_>, Self::Error> {
        let tx = self.pool.begin().await?;
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
    type Error = sqlx::Error;

    async fn commit(self) -> Result<(), Self::Error> {
        self.tx.commit().await
    }
    async fn rollback(self) -> Result<(), Self::Error> {
        self.tx.rollback().await
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
