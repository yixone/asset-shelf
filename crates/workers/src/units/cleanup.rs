use std::time::Duration;

use models::types::MediaId;
use result::Result;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::{
    di::WorkerContext,
    queue::{EventsQueue, TasksSender},
    traits::{AbstractWorker, WorkerConfig},
};

#[derive(Clone)]
pub enum CleanupWorkerTask {
    RemoveMedia(MediaId),
}

pub struct CleanupWorker {
    events: EventsQueue<CleanupWorkerTask>,
    ctx: WorkerContext,
}

impl CleanupWorker {
    pub fn new(ctx: WorkerContext) -> (TasksSender<CleanupWorkerTask>, Self) {
        let queue = EventsQueue::new(1024);
        (queue.tx.clone(), CleanupWorker { events: queue, ctx })
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

        loop {
            tokio::select! {
                Some(r) = self.events.recv() => {
                    todo!()
                }
                _ = cleanup_interval.tick() => {
                    tracing::info!("CleanupWorker: Starting interval auto-cleaning");

                    // TODO
                }
                _ = cancel.cancelled() => {
                    self.events.close();
                    break;
                }
            }
        }

        Ok(())
    }
}
