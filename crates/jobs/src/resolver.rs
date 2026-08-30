use std::sync::Arc;

use flake_id::FlakeIdGenerator;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    JobId, JobQueue, JobSchedule, JobsSnapshot, ResolverHandle, WorkerContext, job::Job,
    schedule::Schedule, worker::AsyncWorker,
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

    /// Application cancellation token
    cancel: CancellationToken,
}

impl JobsResolver {
    /// Creates a new [`JobsResolver`]
    pub fn new(
        workers: usize,
        scheduled: Vec<(Job, JobSchedule)>,
        flake: Arc<FlakeIdGenerator>,
        cancel: CancellationToken,
    ) -> Self {
        let resolver = JobsResolver {
            queue: Arc::new(JobQueue::new(flake.clone())),
            workers_count: workers,
            schedule: Arc::new(Schedule::new(&scheduled)),
            cancel,
        };

        for (j, _) in scheduled.into_iter().filter(|(_, s)| s.is_interval()) {
            resolver.queue(j);
        }

        resolver
    }

    pub fn run(mut self, ctx: Arc<WorkerContext>) -> ResolverHandle {
        // The approximate capacity is indicated based on the calculation: workers + 1 scheduler
        let mut handles = Vec::with_capacity(self.workers_count + 1);

        handles.push(self.run_scheduler());
        handles.extend(self.run_workers(ctx));

        let resolver = Arc::new(self);

        ResolverHandle { resolver, handles }
    }

    /// Returns the snapshot of this [`JobsResolver`]
    pub fn snapshot(&self) -> JobsSnapshot {
        JobsSnapshot {
            schedule: self.schedule.snapshot(),
            queue: self.queue.snapshot(),
        }
    }

    /// Queues one job
    pub fn queue(&self, job: Job) {
        self.queue.queue(job);
    }

    /// Returns the number of running background jobs
    pub fn active_count(&self) -> usize {
        self.queue.active_count()
    }

    /// Cancels and terminates the active job
    pub fn terminate_active(&self, id: JobId) -> bool {
        self.queue.terminate_active(id)
    }

    /// Removes the job with the specified [`JobId`] from the queue
    pub fn remove_queued(&self, id: JobId) -> bool {
        self.queue.remove_queued(id)
    }

    /// Starts all resolver workers
    pub fn run_workers(&mut self, ctx: Arc<WorkerContext>) -> Vec<JoinHandle<()>> {
        let range = 0..self.workers_count;

        let mut handles = Vec::with_capacity(self.workers_count);
        for w in range.map(|_| AsyncWorker::new(self.queue.clone(), ctx.clone())) {
            handles.push(w.worker_loop(self.cancel.clone()));
        }

        handles
    }

    /// Starts the background scheduler for this [`JobsResolver`]
    pub fn run_scheduler(&mut self) -> JoinHandle<()> {
        let queue = self.queue.clone();
        let cancel = self.cancel.clone();
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

    /// Removes all tasks from the queue
    pub(crate) fn clear_queue(&self) -> usize {
        self.queue.clear()
    }
}
