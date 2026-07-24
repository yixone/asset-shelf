use std::sync::Arc;

use db::sqlite::SqliteDatabase;
use flake_id::FlakeIdGenerator;
use storage::Storage;

#[derive(Clone)]
pub struct WorkerContext {
    pub db: Arc<SqliteDatabase>,
    pub flake: Arc<FlakeIdGenerator>,
    pub storage: Arc<Storage>,
}
