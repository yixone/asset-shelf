use std::sync::Arc;

use db::sqlite::SqliteDb;
use flake_id::FlakeIdGenerator;
use storage::Storage;

#[derive(Clone)]
pub struct WorkerContext {
    pub db: Arc<SqliteDb>,
    pub flake: Arc<FlakeIdGenerator>,
    pub storage: Arc<Storage>,
}
