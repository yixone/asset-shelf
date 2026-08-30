use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::{Job, JobId, JobsSnapshot, resolver::JobsResolver};

/// Handle for managing background jobs via [`JobsResolver`]
#[derive(Clone)]
pub struct JobsHandle {
    resolver: Arc<JobsResolver>,
}

impl JobsHandle {
    /// Adds a [`Job`] to the jobs queue
    pub fn queue(&self, job: Job) {
        self.resolver.queue(job)
    }

    /// Cancels the [`Job`] with the specified [`JobId`]
    ///
    /// Cancels the job if it is already running
    pub fn cancel_queued(&self, id: JobId) -> bool {
        if self.resolver.terminate_active(id) {
            return true;
        }

        if self.resolver.remove_queued(id) {
            return true;
        }

        false
    }

    /// Returns the snapshot of [`JobsResolver`]
    pub fn snapshot(&self) -> JobsSnapshot {
        self.resolver.snapshot()
    }

    /// Returns the number of running background jobs
    pub fn active_count(&self) -> usize {
        self.resolver.active_count()
    }
}

/// [`JobsResolver`] handle
pub struct ResolverHandle {
    pub(crate) resolver: Arc<JobsResolver>,
    pub(crate) handles: Vec<JoinHandle<()>>,
}

impl ResolverHandle {
    /// Returns a [`JobsHandle`] for the current [`JobsResolver`]
    pub fn jobs(&self) -> JobsHandle {
        JobsHandle {
            resolver: self.resolver.clone(),
        }
    }

    /// Removes all tasks from the queue and waits for active tasks to complete
    ///
    /// It will only take effect after the cancellation is signaled via the cancellation token;
    /// otherwise, it will wait indefinitely
    pub async fn close(self) {
        let removed = self.resolver.clear_queue();
        tracing::info!("Stopping background tasks; {removed} tasks removed from the queue");

        for handle in self.handles {
            if let Err(e) = handle.await {
                tracing::error!(err = ?e);
            }
        }
    }
}
