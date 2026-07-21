use std::sync::Arc;

use db::sqlite::SqliteDb;
use storage::Storage;

#[derive(Clone)]
pub struct WorkerContext {
    pub db: SqliteDb,
    pub storage: Arc<Storage>,
}
