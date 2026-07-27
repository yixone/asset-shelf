use std::time::Duration;

use result::Result;
use tokio_util::sync::CancellationToken;

pub struct WorkerConfig {
    /// Service name
    pub name: &'static str,

    /// The time after which the worker will
    /// be automatically restarted after a crash
    pub restart_delay: Duration,

    /// Allow worker restart after a crash
    pub allow_restart: bool,
}

#[async_trait::async_trait]
pub trait AbstractWorker {
    /// Returns the background service configuration
    fn cfg(&self) -> WorkerConfig;

    /// Executes the service runtime
    async fn runtime(&mut self, cancel: CancellationToken) -> Result<()>;
}
