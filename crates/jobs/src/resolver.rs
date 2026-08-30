use std::sync::Arc;

use flake_id::FlakeIdGenerator;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    JobId, JobQueue, JobSchedule, JobsSnapshot, WorkerContext, job::Job, schedule::Schedule,
    worker::AsyncWorker,
};

/// Application background jobs resolver
#[derive(Debug)]
pub struct JobsResolver {
    /// Jobs queue
    queue: Arc<JobQueue>,

    /// Number of workers to launch
    workers_count: usize,

    /// Jobs Execution Schedule
    schedule: Arc<Schedule>,
}

impl JobsResolver {
    /// Creates a new [`JobsResolver`]
    pub fn new(
        workers: usize,
        scheduled: Vec<(Job, JobSchedule)>,
        flake: Arc<FlakeIdGenerator>,
    ) -> Self {
        let resolver = JobsResolver {
            queue: Arc::new(JobQueue::new(flake.clone())),
            workers_count: workers,
            schedule: Arc::new(Schedule::new(&scheduled)),
        };

        for (j, _) in scheduled.into_iter().filter(|(_, s)| s.is_interval()) {
            resolver.queue(j);
        }

        resolver
    }

    /// Launches all child background resolver tasks and
    /// returns a [`ResolverTasksHandle`] for managing the stopping of these tasks
    pub fn run(
        self: &Arc<Self>,
        cancel: CancellationToken,
        ctx: Arc<WorkerContext>,
    ) -> ResolverTasksHandle {
        // The approximate capacity is indicated based on the calculation: workers + 1 scheduler
        let mut handles = Vec::with_capacity(self.workers_count + 1);

        handles.push(self.run_scheduler(cancel.clone()));
        handles.extend(self.run_workers(cancel, ctx));

        ResolverTasksHandle {
            resolver: self.clone(),
            handles,
        }
    }

    /// Starts all resolver workers
    fn run_workers(
        &self,
        cancel: CancellationToken,
        ctx: Arc<WorkerContext>,
    ) -> Vec<JoinHandle<()>> {
        let mut handles = Vec::with_capacity(self.workers_count);
        let range = 0..self.workers_count;
        for w in range.map(|_| AsyncWorker::new(self.queue.clone(), ctx.clone())) {
            handles.push(w.worker_loop(cancel.clone()));
        }

        handles
    }

    /// Starts the background scheduler for this [`JobsResolver`]
    fn run_scheduler(&self, cancel: CancellationToken) -> JoinHandle<()> {
        let queue = self.queue.clone();
        let schedule = self.schedule.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(job) = schedule.next_run() => {
                        queue.queue(job);
                    }
                    _ = cancel.cancelled() => {
                        return;
                    }
                }
            }
        })
    }

    /// Queues one job
    pub fn queue(&self, job: Job) {
        self.queue.queue(job);
    }

    /// Cancels the [`Job`] with the specified [`JobId`]
    ///
    /// Cancels the job if it is already running
    pub fn cancel(&self, id: JobId) -> bool {
        if self.terminate_active(id) {
            return true;
        }

        if self.queue.remove_queued(id) {
            return true;
        }

        false
    }

    /// Cancels and terminates the active job
    pub fn terminate_active(&self, id: JobId) -> bool {
        self.queue.terminate_active(id)
    }

    /// Returns the number of running background jobs
    pub fn active_count(&self) -> usize {
        self.queue.active_count()
    }

    /// Returns the workers count of this [`JobsResolver`]
    pub fn workers_count(&self) -> usize {
        self.workers_count
    }

    /// Returns the snapshot of this [`JobsResolver`]
    pub fn snapshot(&self) -> JobsSnapshot {
        JobsSnapshot {
            schedule: self.schedule.snapshot(),
            queue: self.queue.snapshot(),
        }
    }

    /// Removes all tasks from the queue
    pub(crate) fn clear_queue(&self) -> usize {
        self.queue.clear()
    }
}

/// [`JobsResolver`] spawned tasks handle
pub struct ResolverTasksHandle {
    resolver: Arc<JobsResolver>,
    handles: Vec<JoinHandle<()>>,
}

impl ResolverTasksHandle {
    /// Removes all jobs from the queue and waits for active jobs to complete
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
