use std::sync::Arc;

use flake_id::FlakeIdGenerator;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    Job, JobSchedule, JobsSnapshot,
    resolver::JobsResolver,
    worker::{AsyncWorker, WorkerContext},
};

#[derive(Debug, Clone)]
pub struct JobsHandle(Arc<JobsResolver>);

impl JobsHandle {
    /// Adds a [`Job`] to the jobs queue
    pub async fn queue(&self, job: Job) {
        self.0.queue(job).await
    }

    /// Returns a snapshot of the background jobs queue
    pub async fn snapshot(&self) -> JobsSnapshot {
        self.0.snapshot().await
    }
}

pub struct JobsManagerHandle {
    resolver: Arc<JobsResolver>,
    async_handles: Vec<JoinHandle<()>>,
}

impl JobsManagerHandle {
    /// Removes all tasks from the queue and waits for active tasks to complete
    ///
    /// It will only take effect after the cancellation is signaled via the cancellation token;
    /// otherwise, it will wait indefinitely
    pub async fn stop(self) {
        let removed = self.resolver.clear_pending().await;
        tracing::info!("Stopping background tasks; {removed} tasks removed from the queue");

        for ah in self.async_handles {
            if let Err(e) = ah.await {
                tracing::error!(err = ?e);
            }
        }
    }
}

pub struct JobsManager {
    async_workers: Vec<AsyncWorker>,
    resolver: Arc<JobsResolver>,
}

impl JobsManager {
    /// Creates a new [`JobsManager`]
    pub async fn new(
        workers_count: usize,
        scheduled: Vec<(Job, JobSchedule)>,
        ctx: Arc<WorkerContext>,
        flake: Arc<FlakeIdGenerator>,
    ) -> Self {
        let resolver = Arc::new(JobsResolver::new(scheduled, flake.clone()).await);
        let async_workers = (0..workers_count)
            .map(|_| AsyncWorker::new(resolver.clone(), ctx.clone()))
            .collect::<Vec<_>>();

        Self {
            async_workers,
            resolver,
        }
    }

    pub fn run(self, cancel: CancellationToken) -> (JobsHandle, JobsManagerHandle) {
        let mut async_handles = Vec::new();

        let workers_count = self.async_workers.len();

        for aw in self.async_workers {
            async_handles.push(aw.worker_loop(cancel.clone()));
        }
        tracing::info!("   |- Started {workers_count} background workers");

        async_handles.push(self.resolver.run_scheduler(cancel.clone()));
        tracing::info!("   |- Started jobs scheduler");

        let queue_handle = JobsHandle(self.resolver.clone());
        let manager_handle = JobsManagerHandle {
            resolver: self.resolver,
            async_handles,
        };

        (queue_handle, manager_handle)
    }
}
