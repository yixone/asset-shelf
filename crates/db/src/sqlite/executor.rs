use crate::{
    database::DatabaseSession,
    sqlite::{SqliteSession, SqliteTransaction},
};

/// Abstract executor for Sqltie database
pub(crate) trait SqliteExecutor {
    /// Returns the executor for the current session.
    fn executor(&mut self) -> &mut sqlx::SqliteConnection;
}

// Blanket implementation for SQLite sessions
impl<T> DatabaseSession for T where T: SqliteExecutor {}

impl SqliteExecutor for SqliteTransaction {
    fn executor(&mut self) -> &mut sqlx::SqliteConnection {
        &mut self.tx
    }
}

impl SqliteExecutor for SqliteSession {
    fn executor(&mut self) -> &mut sqlx::SqliteConnection {
        &mut self.conn
    }
}
