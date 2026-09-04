use std::{sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    JobsDispatcher,
    dispatcher::ActiveJobPermit,
    job::{BackgroundJob, ExecutionStatus},
};

const WORKER_RESART_DELAY: Duration = Duration::from_mins(2);

/// Asynchronous background worker
pub struct AsyncWorker<J>
where
    J: BackgroundJob,
{
    dispatcher: Arc<JobsDispatcher<J>>,
    ctx: Arc<J::Context>,
}

impl<J> AsyncWorker<J>
where
    J: BackgroundJob + 'static,
{
    /// Creates a new [`AsyncWorker`]
    pub fn new(dispatcher: Arc<JobsDispatcher<J>>, ctx: Arc<J::Context>) -> Self {
        AsyncWorker { dispatcher, ctx }
    }

    pub fn worker_loop(mut self, cancel: CancellationToken) -> JoinHandle<()> {
        tokio::spawn(async move {
            while (self.jobs_loop(cancel.clone()).await).is_err() {
                tokio::time::sleep(WORKER_RESART_DELAY).await;
            }
        })
    }

    async fn jobs_loop(&mut self, cancel: CancellationToken) -> Result<(), J::Error> {
        loop {
            tokio::select! {
                next_job = self.dispatcher.next() => {
                    if let Err(e) = self.perform_job(&next_job).await {
                        tracing::error!(err = ?e, job = ?next_job.inner().job(), "Worker error occurred");

                        if J::can_retry(&e) {
                            next_job.requeue();
                        } else {
                            next_job.mark_executed(ExecutionStatus::Failed);
                        }

                        if J::need_cooldown(&e) {
                            return Err(e);
                        }

                        continue;
                    }

                    next_job.mark_executed(ExecutionStatus::Success);
                },
                _ = cancel.cancelled() => return Ok(()),
            }
        }
    }

    async fn perform_job(&mut self, permit: &ActiveJobPermit<J>) -> Result<(), J::Error> {
        let job = permit.inner();

        tokio::select! {
            _ = job.cancelled() => {
                Ok(())
            }
            res = job.job().execute(&self.ctx) => {
                res
            }
        }
    }
}
