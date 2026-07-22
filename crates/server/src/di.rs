use std::sync::Arc;

use db::sqlite::SqliteDb;
use storage::Storage;
use workers::{queue::TasksSender, units::cleanup::CleanupWorkerTask};

pub struct DataCtx {
    pub db: Arc<SqliteDb>,
    pub storage: Arc<Storage>,
}

pub struct EventsContext {
    pub cleanup: TasksSender<CleanupWorkerTask>,
}
