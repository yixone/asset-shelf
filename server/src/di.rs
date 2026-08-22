use std::sync::Arc;

use config::ApplicationConfig;
use db::RepositoryContext;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use result::Result;
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

/// Application metrics context
pub struct MetricsCtx {
    /// Global registry of Metrics
    pub registry: MetricsRegistry,

    /// Server instance metrics
    pub server: ServerMetrics,
}

impl MetricsCtx {
    /// Tries to create a new [`MetricsCtx`]
    pub fn try_new(reg: MetricsRegistry) -> Result<Self> {
        Ok(MetricsCtx {
            server: ServerMetrics::try_new(&reg)?,
            registry: reg,
        })
    }
}
