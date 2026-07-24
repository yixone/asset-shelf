use std::sync::Arc;

use db::sqlite::SqliteDatabase;
use flake_id::FlakeIdGenerator;
use storage::Storage;
use workers::{
    queue::TasksSender,
    units::{cleanup::CleanupWorkerTask, media::MediaWorkerTask},
};

pub struct DataCtx {
    pub db: Arc<SqliteDatabase>,
    pub storage: Arc<Storage>,
    pub flake: Arc<FlakeIdGenerator>,
}

pub struct EventsContext {
    pub cleanup: TasksSender<CleanupWorkerTask>,
    pub media: TasksSender<MediaWorkerTask>,
}
