use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use flake_id::FlakeIdGenerator;

use crate::{
    job::{ActiveJob, Job, JobId},
    queue::JobsQueue,
};

/// Application background jobs resolver
#[derive(Debug)]
pub struct JobsResolver {
    /// Jobs queue
    queue: JobsQueue,

    /// Resolver runtime data
    runtime: Mutex<JobsResolverRuntime>,

    /// ID generator
    flake: Arc<FlakeIdGenerator>,
}

#[derive(Debug)]
struct JobsResolverRuntime {
    /// Scheduled jobs kinds
    scheduled_kinds: HashSet<&'static str>,

    /// Active jobs currently being processed
    active: HashMap<JobId, Arc<ActiveJob>>,
}

impl JobsResolver {
    /// Creates a new [`JobsResolver`]
    pub fn new(flake: Arc<FlakeIdGenerator>) -> Self {
        Self {
            queue: JobsQueue::new(),
            runtime: Mutex::new(JobsResolverRuntime {
                scheduled_kinds: HashSet::new(),
                active: HashMap::new(),
            }),
            flake,
        }
    }

    /// Queues one job
    pub async fn queue(&self, job: Job) {
        if !job.allow_concurrency() {
            let kind = job.kind();

            let mut lock = self.runtime.lock().unwrap_or_else(|i| i.into_inner());
            if !lock.scheduled_kinds.insert(kind) {
                return;
            }
        }

        let id = self.flake.get_id_as();
        self.queue.queue((id, job)).await;
    }

    /// Returns a new job from the queue, or otherwise waits for new jobs to be added to the queue
    pub async fn next(self: &Arc<Self>) -> ActiveJobPermit {
        let (id, job) = self.queue.pop().await;
        let active = Arc::new(ActiveJob::new(job));

        {
            let mut lock = self.runtime.lock().unwrap_or_else(|i| i.into_inner());
            lock.active.insert(id, active.clone());
        }

        ActiveJobPermit {
            resolver: self.clone(),
            job_id: id,
            inner: active,
        }
    }

    /// Removes all jobs from the queue and returns the total number of removed elements
    pub async fn drain(&self) -> usize {
        self.queue.drain().await
    }

    /// Returns a snapshot of the background jobs queue
    pub async fn snapshot(&self) -> Vec<(JobId, &'static str)> {
        self.queue.snapshot().await
    }

    /// Removes the job from the active list
    fn remove_active(&self, id: JobId, job: &Job) {
        let mut lock = self.runtime.lock().unwrap_or_else(|i| i.into_inner());
        lock.active.remove(&id);
        if !job.allow_concurrency() {
            lock.scheduled_kinds.remove(job.kind());
        }
    }
}

/// Permit for active job
///
/// Upon a Drop, the job is automatically removed from the list of active jobs
pub struct ActiveJobPermit {
    /// The resolver that created this permit
    resolver: Arc<JobsResolver>,
    job_id: JobId,
    inner: Arc<ActiveJob>,
}

impl ActiveJobPermit {
    /// Returns a reference to the job of this [`ActiveJobPermit`]
    pub fn inner(&self) -> &ActiveJob {
        &self.inner
    }
}

impl Drop for ActiveJobPermit {
    fn drop(&mut self) {
        self.resolver.remove_active(self.job_id, self.inner.job());
    }
}
