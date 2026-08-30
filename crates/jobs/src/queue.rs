use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex, MutexGuard},
};

use flake_id::FlakeIdGenerator;
use tokio::sync::Notify;

use crate::{
    Job, JobsQueueSnapshot,
    job::{ActiveJob, JobId},
};

#[derive(Debug)]
pub struct JobQueue {
    inner: Mutex<JobsQueueInner>,
    notify: Notify,
    flake: Arc<FlakeIdGenerator>,
}

impl JobQueue {
    /// Creates a new [`JobQueue`]
    pub fn new(flake: Arc<FlakeIdGenerator>) -> Self {
        Self {
            inner: Mutex::new(JobsQueueInner::new()),
            notify: Notify::new(),
            flake,
        }
    }

    /// Returns the mutex guard for the jobs queue contents
    fn lock<'a>(&'a self) -> MutexGuard<'a, JobsQueueInner> {
        self.inner.lock().unwrap_or_else(|f| f.into_inner())
    }

    /// Returns the number of running background jobs
    pub fn active_count(&self) -> usize {
        let lock = self.lock();
        lock.active.len()
    }

    /// Returns the snapshot of this [`JobQueue`]
    pub fn snapshot(&self) -> JobsQueueSnapshot {
        let lock = self.lock();
        JobsQueueSnapshot {
            active: lock
                .active
                .iter()
                .map(|(&id, j)| (id, j.job().clone()))
                .collect(),
            queue: lock.queue.iter().cloned().collect(),
        }
    }

    /// Adds a new task to the queue
    pub fn queue(&self, job: Job) -> Option<JobId> {
        let mut lock = self.lock();

        if !job.allow_concurrency() && !lock.pending_kinds.insert(job.kind()) {
            return None;
        }

        let id = self.flake.get_id_as();
        let kind = job.kind();

        lock.queue.push_back((id, job));
        drop(lock);

        tracing::info!("New job added: {id}: {kind}");

        self.notify.notify_one();

        Some(id)
    }

    /// Removes all pending tasks from the queue
    pub fn clear(&self) -> usize {
        let mut lock = self.lock();
        lock.pending_kinds.clear();
        lock.queue.drain(..).count()
    }

    /// Returns a next job from the queue
    pub async fn next_job(self: &Arc<Self>) -> ActiveJobPermit {
        let (id, job) = self.pool_queue().await;
        let active = Arc::new(ActiveJob::new(job));

        {
            let mut lock = self.lock();
            lock.active.insert(id, active.clone());
        }

        ActiveJobPermit {
            queue: self.clone(),
            job_id: id,
            inner: active,
            cleanup_on_drop: true,
        }
    }

    /// Cancels and terminates the [`ActiveJob`]
    pub fn terminate_active(&self, id: JobId) -> bool {
        let mut lock = self.lock();
        if let Some(active) = lock.active.remove(&id) {
            active.cancel();

            if !active.job().allow_concurrency() {
                lock.pending_kinds.remove(active.job().kind());
            }

            return true;
        }

        false
    }

    /// Removes the job with the specified [`JobId`] from the queue
    pub fn remove_queued(&self, id: JobId) -> bool {
        let mut lock = self.lock();

        let Ok(idx) = lock.queue.binary_search_by(|q| q.0.cmp(&id)) else {
            return false;
        };

        if let Some((_, job)) = lock.queue.remove(idx) {
            if !job.allow_concurrency() {
                lock.pending_kinds.remove(job.kind());
            }

            return true;
        }

        false
    }

    async fn pool_queue(&self) -> (JobId, Job) {
        loop {
            let permit = self.notify.notified();

            if let Some(j) = {
                let mut lock = self.lock();
                lock.queue.pop_front()
            } {
                return j;
            }

            permit.await;
        }
    }

    fn remove_active(&self, id: JobId) {
        let mut lock = self.lock();
        if let Some(removed) = lock.active.remove(&id)
            && !removed.job().allow_concurrency()
        {
            lock.pending_kinds.remove(removed.job().kind());
        }
    }

    fn requeue(&self, id: JobId) -> bool {
        let mut lock = self.lock();
        let Some(removed) = lock.active.remove(&id) else {
            return false;
        };

        let job = removed.job().clone();

        lock.queue.push_back((id, job));
        drop(lock);

        self.notify.notify_one();

        true
    }
}

#[derive(Debug)]
struct JobsQueueInner {
    queue: VecDeque<(JobId, Job)>,
    active: HashMap<JobId, Arc<ActiveJob>>,
    pending_kinds: HashSet<&'static str>,
}

impl JobsQueueInner {
    /// Creates a new [`JobsQueueInner`]
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            pending_kinds: HashSet::new(),
            active: HashMap::new(),
        }
    }
}

/// Permit for active job
///
/// Upon a Drop, the job is automatically removed from the list of active jobs
pub struct ActiveJobPermit {
    queue: Arc<JobQueue>,
    job_id: JobId,
    inner: Arc<ActiveJob>,

    cleanup_on_drop: bool,
}

impl ActiveJobPermit {
    /// Returns a reference to the job of this [`ActiveJobPermit`]
    pub fn inner(&self) -> &ActiveJob {
        &self.inner
    }

    /// Removes the current task from the list of active tasks and returns it to the end of the queue
    pub fn requeue(mut self) {
        self.queue.requeue(self.job_id);
        self.cleanup_on_drop = false;
    }
}

impl Drop for ActiveJobPermit {
    fn drop(&mut self) {
        if !self.cleanup_on_drop {
            return;
        }

        self.queue.remove_active(self.job_id);
    }
}
