use std::{sync::Arc, time::Duration};

use result::Result;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{JobsDispatcher, dispatcher::ActiveJobPermit, job::BackgroundJob};

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

    async fn jobs_loop(&mut self, cancel: CancellationToken) -> Result<()> {
        loop {
            tokio::select! {
                next_job = self.dispatcher.next() => {
                    if let Err(e) = self.perform_job(&next_job).await {
                        tracing::error!(err = ?e, job = ?next_job.inner().job(), "Worker error occurred");

                        if e.is_retryable() {
                            next_job.requeue();
                        }

                        if e.is_internal() {
                            return Err(e);
                        }
                    }
                },
                _ = cancel.cancelled() => return Ok(()),
            }
        }
    }

    async fn perform_job(&mut self, permit: &ActiveJobPermit<J>) -> Result<()> {
        let job = permit.inner();

        tokio::select! {
            _ = job.cancelled() => {
                Ok(())
            }
            _ = job.job().execute(&self.ctx) => {
                Ok(())
            }
        }
    }
}
