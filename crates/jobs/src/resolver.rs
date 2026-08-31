use std::{num::NonZeroUsize, sync::Arc};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    JobId, JobSchedule, JobsDispatcher, JobsSnapshot, WorkerContext, job::Job, schedule::Schedule,
    worker::AsyncWorker,
};

/// Application background jobs resolver
pub struct JobsResolver {
    /// Jobs dispatcher
    dispatcher: Arc<JobsDispatcher>,

    /// Number of workers to launch
    workers_count: usize,

    /// Jobs Execution Schedule
    schedule: Arc<Schedule>,

    /// Cancellation token for tasks started by the resolver
    cancel: CancellationToken,

    /// Background worker context
    context: Arc<WorkerContext>,
}

impl JobsResolver {
    /// Creates a new [`JobsResolverBuilder`]
    pub fn builder() -> JobsResolverBuilder {
        JobsResolverBuilder {
            dispatcher: None,
            workers_count: get_available_parallelism(),
            cancel: None,
            context: None,
        }
    }

    /// Launches all child background resolver tasks and
    /// returns a [`ResolverTasksHandle`] for managing the stopping of these tasks
    pub fn run(self: &Arc<Self>) -> ResolverTasksHandle {
        // The approximate capacity is indicated based on the calculation: workers + 1 scheduler
        let mut handles = Vec::with_capacity(self.workers_count + 1);

        handles.push(self.run_scheduler(self.cancel.clone()));
        handles.extend(self.run_workers(self.cancel.clone(), self.context.clone()));

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
        for w in range.map(|_| AsyncWorker::new(self.dispatcher.clone(), ctx.clone())) {
            handles.push(w.worker_loop(cancel.clone()));
        }

        handles
    }

    /// Starts the background scheduler for this [`JobsResolver`]
    fn run_scheduler(&self, cancel: CancellationToken) -> JoinHandle<()> {
        let queue = self.dispatcher.clone();
        let schedule = self.schedule.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(job) = schedule.next_run() => {
                        queue.enqueue(job);
                    }
                    _ = cancel.cancelled() => {
                        return;
                    }
                }
            }
        })
    }

    /// Adds a new task to the queue and notifies one waiting worker
    ///
    /// Does not add the task to the queue if it violates concurrency constraints
    pub fn enqueue(&self, job: Job) {
        self.dispatcher.enqueue(job);
    }

    /// Adds new job to the schedule
    pub fn schedule(&self, job: Job, schedule: JobSchedule) {
        self.schedule.schedule_many([(job.clone(), schedule)]);
        if schedule.is_interval() {
            self.enqueue(job);
        }
    }

    /// Cancels the [`Job`] with the specified [`JobId`]
    ///
    /// Cancels the job if it is already running
    pub fn cancel(&self, id: JobId) -> bool {
        if self.terminate_active(id) {
            return true;
        }

        if self.dispatcher.remove_queued(id) {
            return true;
        }

        false
    }

    /// Cancels and terminates the active job
    pub fn terminate_active(&self, id: JobId) -> bool {
        self.dispatcher.terminate_active(id)
    }

    /// Returns the number of running background jobs
    pub fn active_count(&self) -> usize {
        self.dispatcher.active_count()
    }

    /// Returns the workers count of this [`JobsResolver`]
    pub fn workers_count(&self) -> usize {
        self.workers_count
    }

    /// Returns the snapshot of this [`JobsResolver`]
    pub fn snapshot(&self) -> JobsSnapshot {
        JobsSnapshot {
            schedule: self.schedule.snapshot(),
            queue: self.dispatcher.snapshot(),
        }
    }

    /// Removes all tasks from the queue
    pub(crate) fn clear_queue(&self) -> usize {
        self.dispatcher.clear()
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

pub struct JobsResolverBuilder {
    /// Jobs dispatcher
    dispatcher: Option<JobsDispatcher>,

    /// Number of workers to launch
    workers_count: usize,

    /// Cancellation token for tasks started by the resolver
    cancel: Option<CancellationToken>,

    /// Background worker context
    context: Option<WorkerContext>,
}

impl JobsResolverBuilder {
    /// Sets the dispatcher of [`JobsResolver`]
    pub fn dispatcher(mut self, dispatcher: JobsDispatcher) -> Self {
        self.dispatcher = Some(dispatcher);
        self
    }

    /// Sets the workers count of [`JobsResolver`]
    pub fn workers_count(mut self, count: usize) -> Self {
        self.workers_count = count;
        self
    }

    /// Sets the cancel of [`JobsResolver`]
    pub fn cancel(mut self, cancel: CancellationToken) -> Self {
        self.cancel = Some(cancel);
        self
    }

    /// Sets the context of [`JobsResolver`]
    pub fn context(mut self, ctx: WorkerContext) -> Self {
        self.context = Some(ctx);
        self
    }

    /// Builds the resolver or returns a missing-fields error
    pub fn build(self) -> Result<JobsResolver, MissingBuilderFields> {
        self.try_build().ok_or(MissingBuilderFields)
    }

    pub fn build_shared(self) -> Result<Arc<JobsResolver>, MissingBuilderFields> {
        self.build().map(Arc::new)
    }

    fn try_build(self) -> Option<JobsResolver> {
        Some(JobsResolver {
            dispatcher: Arc::new(self.dispatcher?),
            workers_count: self.workers_count,
            schedule: Arc::new(Schedule::new()),
            cancel: self.cancel?,
            context: Arc::new(self.context?),
        })
    }
}

#[derive(Debug)]
pub struct MissingBuilderFields;

impl std::fmt::Display for MissingBuilderFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing jobs resolver builder fields")
    }
}

impl std::error::Error for MissingBuilderFields {}

/// Returns the maximum number of parallel threads
fn get_available_parallelism() -> usize {
    std::thread::available_parallelism().map_or(2, NonZeroUsize::get)
}
