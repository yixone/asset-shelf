use std::sync::Arc;

use db::RepositoryContext;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use storage::Storage;

pub mod supervisor;
pub mod worker;

pub mod units;

pub mod cleanup;
pub mod media;

#[derive(Clone)]
pub struct WorkerContext {
    pub db: Arc<RepositoryContext>,
    pub flake: Arc<FlakeIdGenerator>,
    pub storage: Arc<Storage>,
    pub events: Arc<EventBus>,
}
