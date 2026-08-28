use std::{sync::Arc, time::Duration};

use db::RepositoryContext;
use flake_id::FlakeIdGenerator;
use result::Result;
use storage::Storage;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::resolver::{ActiveJobPermit, JobsResolver};

const WORKER_RESART_DELAY: Duration = Duration::from_mins(2);

/// Shared context of the workers
#[derive(Clone)]
pub struct WorkerContext {
    pub db: Arc<RepositoryContext>,
    pub flake: Arc<FlakeIdGenerator>,
    pub storage: Arc<Storage>,
}

/// Asynchronous background worker
pub struct AsyncWorker(Arc<JobsResolver>, Arc<WorkerContext>);

impl AsyncWorker {
    /// Creates a new [`AsyncWorker`]
    pub fn new(resolver: Arc<JobsResolver>, ctx: Arc<WorkerContext>) -> Self {
        AsyncWorker(resolver, ctx)
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
                next_job = self.0.next() => {
                    if let Err(e) = self.perform_job(&next_job).await {
                        tracing::error!(err = ?e, job = ?next_job.inner().job(), "Worker error occurred");

                        if e.is_internal() {
                            return Err(e);
                        }
                    }
                },
                _ = cancel.cancelled() => return Ok(()),
            }
        }
    }

    async fn perform_job(&mut self, job: &ActiveJobPermit) -> Result<()> {
        match job.inner().job() {
            crate::Job::ProcessAssetMedia { .. } => {
                dbg!(job.inner().job());
            }
            crate::Job::CleanupStorageMedia => {
                dbg!(job.inner().job());
            }
        }
        Ok(())
    }
}
