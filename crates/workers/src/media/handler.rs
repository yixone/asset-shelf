use std::time::Duration;

use events::AssetCreatedEvent;
use result::Result;
use tokio_util::sync::CancellationToken;

use crate::{
    media::jobs,
    runtime::{AbstractWorker, WorkerConfig, WorkerContext},
};

pub struct MediaWorker {
    ctx: WorkerContext,
}

impl MediaWorker {
    /// Creates a new [`MediaWorker`]
    pub fn new(ctx: WorkerContext) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl AbstractWorker for MediaWorker {
    /// Returns the background service configuration
    fn cfg(&self) -> WorkerConfig {
        WorkerConfig {
            name: "MediaWorker",
            restart_delay: Duration::from_secs(30),
            allow_restart: true,
        }
    }

    /// Executes the service runtime
    async fn runtime(&mut self, cancel: CancellationToken) -> Result<()> {
        jobs::process::process_unprocessed_media(&self.ctx).await?;

        let mut on_asset_created = self.ctx.events.subscribe::<AssetCreatedEvent>();

        loop {
            tokio::select! {
                Ok(new_asset) = on_asset_created.recv() => {
                    jobs::process::process_asset_by_id(&self.ctx, new_asset.asset_id).await?;
                },
                _ = cancel.cancelled() => break
            }
        }

        Ok(())
    }
}
