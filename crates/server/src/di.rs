use std::sync::Arc;

use db::sqlite::SqliteDb;
use flake_id::FlakeIdGenerator;
use storage::Storage;
use workers::{queue::TasksSender, units::cleanup::CleanupWorkerTask};

pub struct DataCtx {
    pub db: Arc<SqliteDb>,
    pub storage: Arc<Storage>,
    pub flake: Arc<FlakeIdGenerator>,
}

pub struct EventsContext {
    pub cleanup: TasksSender<CleanupWorkerTask>,
}
