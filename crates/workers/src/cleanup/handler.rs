use std::time::Duration;

use events::AssetDeletedEvent;
use result::Result;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::{
    cleanup::jobs,
    runtime::{AbstractWorker, WorkerConfig, WorkerContext},
};

pub struct CleanupWorker {
    ctx: WorkerContext,
}

impl CleanupWorker {
    /// Creates a new [`CleanupWorker`]
    pub fn new(ctx: WorkerContext) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl AbstractWorker for CleanupWorker {
    /// Returns the background service configuration
    fn cfg(&self) -> WorkerConfig {
        WorkerConfig {
            name: "CleanupWorker",
            restart_delay: Duration::from_secs(30),
            allow_restart: true,
        }
    }

    /// Executes the service runtime
    async fn runtime(&mut self, cancel: CancellationToken) -> Result<()> {
        let mut cleanup_interval = interval(Duration::from_mins(90));

        let mut on_asset_removed = self.ctx.events.subscribe::<AssetDeletedEvent>();

        loop {
            tokio::select! {
                Ok(e) = on_asset_removed.recv() => {
                    jobs::remove::remove_media_by_id(&self.ctx, &e.media).await?
                }
                _ = cleanup_interval.tick() => {
                    tracing::info!("CleanupWorker: Starting interval auto-cleaning");

                    let orphaned = jobs::cleanup::cleanup_orphaned(&self.ctx).await?;
                    tracing::info!("CleanupWorker: {orphaned} orphaned Media removed");

                    let del_assets = jobs::cleanup::cleanup_deleted_assets(&self.ctx).await?;
                    tracing::info!("CleanupWorker: {del_assets} assets marked as deleted have been deleted");
                }
                _ = cancel.cancelled() => {
                    break;
                }
            }
        }

        Ok(())
    }
}
