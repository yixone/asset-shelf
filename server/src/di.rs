use std::sync::Arc;

use db::RepositoryContext;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use storage::Storage;

pub struct DataCtx {
    pub db: Arc<RepositoryContext>,
    pub storage: Arc<Storage>,
    pub flake: Arc<FlakeIdGenerator>,
    pub events: Arc<EventBus>,
}
