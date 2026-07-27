use std::sync::Arc;

use db::sqlite::SqliteDatabase;
use flake_id::FlakeIdGenerator;
use storage::Storage;
use workers::events::EventBus;

pub struct DataCtx {
    pub db: Arc<SqliteDatabase>,
    pub storage: Arc<Storage>,
    pub flake: Arc<FlakeIdGenerator>,
    pub events: Arc<EventBus>,
}
