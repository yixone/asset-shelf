use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::{Job, resolver::JobsResolver};

#[derive(Clone)]
pub struct JobsHandle(Arc<JobsResolver>);

impl JobsHandle {
    /// Adds a [`Job`] to the jobs queue
    pub fn queue(&self, job: Job) {
        self.0.queue(job)
    }
}

/// Task resolver handle
pub struct ResolverHandle {
    pub(crate) resolver: Arc<JobsResolver>,
    pub(crate) handles: Vec<JoinHandle<()>>,
}

impl ResolverHandle {
    /// Returns a [`JobsHandle`] for the current [`JobsResolver`]
    pub fn jobs(&self) -> JobsHandle {
        JobsHandle(self.resolver.clone())
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

// pub struct JobsManager {
//     async_workers: Vec<AsyncWorker>,
//     resolver: Arc<JobsResolver>,
// }

// impl JobsManager {
//     /// Creates a new [`JobsManager`]
//     pub async fn new(
//         workers_count: usize,
//         scheduled: Vec<(Job, JobSchedule)>,
//         ctx: Arc<WorkerContext>,
//         flake: Arc<FlakeIdGenerator>,
//     ) -> Self {
//         todo!()
//     }

//     pub fn run(self, cancel: CancellationToken) -> (JobsHandle, JobsManagerHandle) {
//          let mut async_handles = Vec::new();

//          let workers_count = self.async_workers.len();

//          for aw in self.async_workers {
//              async_handles.push(aw.worker_loop(cancel.clone()));
//          }
//          tracing::info!("   |- Started {workers_count} background workers");

//          async_handles.push(self.resolver.run_scheduler(cancel.clone()));
//          tracing::info!("   |- Started jobs scheduler");

//          let queue_handle = JobsHandle(self.resolver.clone());
//          let manager_handle = JobsManagerHandle {
//              resolver: self.resolver,
//              async_handles,
//          };

//          (queue_handle, manager_handle)

//         todo!()
//     }
// }
