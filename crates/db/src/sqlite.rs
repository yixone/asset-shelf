use sqlx::{
    Sqlite, SqliteConnection, SqlitePool, SqliteTransaction, pool::PoolConnection,
    sqlite::SqliteQueryResult,
};

use crate::{
    core::result::{DeleteResult, InsertResult, UpdateResult},
    provider::{ConnectionUnit, DbProvider, ExecutorUnit, Provider, TransactionUnit},
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

impl Provider for SqliteDb {
    type Connection = SqliteConn;
    type Transaction<'a> = SqliteTx<'a>;
}

impl DbProvider for SqliteDb {
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

impl ExecutorUnit for SqliteTx<'_> {
    type Executor = SqliteConnection;
    fn exec(&mut self) -> &mut Self::Executor {
        &mut self.tx
    }
}

impl ExecutorUnit for SqliteConn {
    type Executor = SqliteConnection;
    fn exec(&mut self) -> &mut Self::Executor {
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
