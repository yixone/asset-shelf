use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex, MutexGuard},
};

use flake_id::FlakeIdGenerator;
use tokio::sync::Notify;

use crate::{
    Job,
    job::{ActiveJob, JobId},
};

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

    /// Adds a new task to the queue
    pub fn queue(&self, job: Job) -> Option<JobId> {
        let mut lock = self.lock();

        if !job.allow_concurrency() && !lock.add_pending(&job) {
            return None;
        }

        let id = self.flake.get_id_as();

        lock.push_job(id, job);
        drop(lock);

        self.notify.notify_one();

        Some(id)
    }

    /// Removes all pending tasks from the queue
    pub fn clear(&self) -> usize {
        let mut lock = self.lock();
        lock.clear()
    }

    /// Returns a next job from the queue
    pub async fn next_job(self: &Arc<Self>) -> ActiveJobPermit {
        let (id, job) = self.pool_queue().await;
        let active = Arc::new(ActiveJob::new(job));

        {
            let mut lock = self.lock();
            lock.insert_active(id, active.clone());
        }

        ActiveJobPermit {
            queue: self.clone(),
            job_id: id,
            inner: active,
        }
    }

    /// Returns a new job from the queue, or waits for a new job to appear if the queue is empty
    async fn pool_queue(&self) -> (JobId, Job) {
        loop {
            let permit = self.notify.notified();

            if let Some(j) = {
                let mut lock = self.lock();
                lock.pop_queue()
            } {
                return j;
            }

            permit.await;
        }
    }

    fn remove_active(&self, id: JobId) {
        let mut lock = self.lock();
        if let Some(removed) = lock.remove_active(id)
            && !removed.job().allow_concurrency()
        {
            lock.remove_pending(removed.job());
        }
    }
}

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

    fn push_job(&mut self, id: JobId, job: Job) {
        self.queue.push_back((id, job));
    }

    fn clear(&mut self) -> usize {
        self.pending_kinds.clear();

        self.queue.drain(..).count()
    }

    fn insert_active(&mut self, id: JobId, job: Arc<ActiveJob>) {
        self.active.insert(id, job);
    }

    fn pop_queue(&mut self) -> Option<(JobId, Job)> {
        self.queue.pop_front()
    }

    fn remove_active(&mut self, id: JobId) -> Option<Arc<ActiveJob>> {
        self.active.remove(&id)
    }

    fn add_pending(&mut self, job: &Job) -> bool {
        self.pending_kinds.insert(job.kind())
    }

    fn remove_pending(&mut self, job: &Job) -> bool {
        self.pending_kinds.remove(job.kind())
    }
}

/// Permit for active job
///
/// Upon a Drop, the job is automatically removed from the list of active jobs
pub struct ActiveJobPermit {
    queue: Arc<JobQueue>,
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
        self.queue.remove_active(self.job_id);
    }
}
