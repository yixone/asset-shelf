use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use flake_id::FlakeIdGenerator;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    JobSchedule, ScheduledJob,
    job::{ActiveJob, Job, JobId},
    queue::JobsQueue,
    schedule::Schedule,
};

/// Application background jobs resolver
#[derive(Debug)]
pub struct JobsResolver {
    /// Jobs queue
    queue: JobsQueue,

    /// Resolver runtime data
    runtime: Mutex<JobsResolverRuntime>,

    /// Jobs Execution Schedule
    schedule: Schedule,

    /// ID generator
    flake: Arc<FlakeIdGenerator>,
}

#[derive(Debug)]
struct JobsResolverRuntime {
    /// Pending jobs kinds
    pending_kinds: HashSet<&'static str>,

    /// Active jobs currently being processed
    active: HashMap<JobId, Arc<ActiveJob>>,
}

impl JobsResolver {
    /// Creates a new [`JobsResolver`]
    pub async fn new(scheduled: Vec<(Job, JobSchedule)>, flake: Arc<FlakeIdGenerator>) -> Self {
        let resolver = JobsResolver {
            queue: JobsQueue::new(),
            runtime: Mutex::new(JobsResolverRuntime {
                pending_kinds: HashSet::new(),
                active: HashMap::new(),
            }),
            schedule: Schedule::new(&scheduled),
            flake,
        };

        for (j, _) in scheduled.into_iter().filter(|(_, s)| s.is_interval()) {
            resolver.queue(j).await;
        }

        resolver
    }

    /// Queues one job
    pub async fn queue(&self, job: Job) {
        if !job.allow_concurrency() {
            let kind = job.kind();

            let mut lock = self.runtime.lock().unwrap_or_else(|i| i.into_inner());
            if !lock.pending_kinds.insert(kind) {
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

    /// Removes all pending tasks from the queue and returns the total number of removed elements
    pub async fn clear_pending(&self) -> usize {
        {
            let mut lock = self.runtime.lock().unwrap_or_else(|i| i.into_inner());
            lock.pending_kinds.drain();
        }
        self.queue.drain().await
    }

    /// Returns a snapshot of the background jobs queue
    pub async fn snapshot(&self) -> JobsSnapshot {
        let queue = self.queue.snapshot().await;
        let active = {
            let lock = self.runtime.lock().unwrap_or_else(|i| i.into_inner());
            lock.active
                .iter()
                .map(|(k, v)| (*k, v.job().kind()))
                .collect::<Vec<_>>()
        };
        let schedule = self.schedule.snapshot().await;

        JobsSnapshot {
            queue,
            active,
            schedule,
        }
    }

    /// Removes the job from the active list
    fn remove_active(&self, id: JobId) {
        let mut lock = self.runtime.lock().unwrap_or_else(|i| i.into_inner());
        if let Some(removed) = lock.active.remove(&id)
            && !removed.job().allow_concurrency()
        {
            lock.pending_kinds.remove(removed.job().kind());
        }
    }

    /// Starts the background scheduler for this [`JobsResolver`]
    pub fn run_scheduler(self: &Arc<Self>, cancel: CancellationToken) -> JoinHandle<()> {
        let resolver = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(job) = resolver.schedule.next_run() => {
                        resolver.queue(job).await;
                    }
                    _ = cancel.cancelled() => {
                        return;
                    }
                }
            }
        })
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
        self.resolver.remove_active(self.job_id);
    }
}

/// Resolver jobs snapshot
#[derive(Debug)]
pub struct JobsSnapshot {
    pub queue: Vec<(JobId, &'static str)>,
    pub active: Vec<(JobId, &'static str)>,
    pub schedule: Vec<ScheduledJob>,
}
