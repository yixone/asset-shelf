use std::sync::Arc;

use db::sqlite::SqliteDatabase;
use flake_id::FlakeIdGenerator;
use storage::Storage;

use crate::events::bus::EventBus;

pub mod supervisor;
pub mod traits;

pub mod events;
pub mod units;

#[derive(Clone)]
pub struct WorkerContext {
    pub db: Arc<SqliteDatabase>,
    pub flake: Arc<FlakeIdGenerator>,
    pub storage: Arc<Storage>,
    pub events: Arc<EventBus>,
}
