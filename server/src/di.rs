use std::sync::Arc;

use config::ApplicationConfig;
use db::RepositoryContext;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use storage::Storage;
use telemetry::MetricsRegistry;

use crate::metrics::ServerMetrics;

pub struct DataCtx {
    pub db: Arc<RepositoryContext>,
    pub storage: Arc<Storage>,
    pub flake: Arc<FlakeIdGenerator>,
    pub events: Arc<EventBus>,

    pub config: Arc<ApplicationConfig>,
}

pub struct MetricsCtx {
    pub registry: MetricsRegistry,

    pub server: ServerMetrics,
}
