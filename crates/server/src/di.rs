use std::sync::Arc;

use db::sqlite::SqliteDb;
use storage::Storage;

pub struct DataCtx {
    pub db: Arc<SqliteDb>,
    pub storage: Arc<Storage>,
}
