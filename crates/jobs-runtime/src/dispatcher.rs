use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex, MutexGuard},
};

use flake_id::FlakeIdGenerator;
use tokio::sync::Notify;

use crate::{
    JobsQueueSnapshot,
    job::{BackgroundJob, JobId},
};

/// Jobs queue broker
///
/// Stores and automatically distributes background jobs among workers
#[derive(Debug)]
pub struct JobsDispatcher<J: BackgroundJob> {
    inner: Mutex<JobsQueueInner<J>>,
    on_new_job: Notify,
    flake: Arc<FlakeIdGenerator>,
}

impl<J: BackgroundJob> JobsDispatcher<J> {
    /// Creates a new [`JobsDispatcher`]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(JobsQueueInner {
                queue: VecDeque::new(),
                active: HashMap::new(),
                pending_kinds: HashSet::new(),
            }),
            on_new_job: Notify::new(),
            flake: Arc::new(FlakeIdGenerator::new(0)),
        }
    }

    /// Returns the mutex guard for the jobs queue contents
    fn lock<'a>(&'a self) -> MutexGuard<'a, JobsQueueInner<J>> {
        self.inner.lock().unwrap_or_else(|f| f.into_inner())
    }

    /// Returns the number of running background jobs
    pub fn active_count(&self) -> usize {
        let lock = self.lock();
        lock.active.len()
    }

    /// Returns the snapshot of this [`JobsDispatcher`]
    pub fn snapshot(&self) -> JobsQueueSnapshot<J> {
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

    /// Adds multiple jobs to the queue, skipping jobs
    /// that violate concurrency constraints, and notifies waiting workers
    pub fn enqueue_many(&self, jobs: impl IntoIterator<Item = J>) -> Vec<JobId> {
        let mut lock = self.lock();
        let mut added = Vec::new();

        for job in jobs {
            if !job.allow_concurrency() && !lock.pending_kinds.insert(job.kind()) {
                continue;
            }

            let id = self.flake.get_id_as();
            lock.queue.push_back((id, job));

            added.push(id);
        }

        drop(lock);

        match added.len() {
            0 => (),
            1 => self.on_new_job.notify_one(),
            _ => self.on_new_job.notify_waiters(),
        }

        added
    }

    /// Adds a new task to the queue and notifies one waiting worker
    ///
    /// Does not add the task to the queue if it violates concurrency constraints
    pub fn enqueue(&self, job: J) -> Option<JobId> {
        self.enqueue_many([job]).into_iter().next()
    }

    /// Removes all pending tasks from the queue
    pub fn clear(&self) -> usize {
        let mut lock = self.lock();
        lock.pending_kinds.clear();
        lock.queue.drain(..).count()
    }

    /// Returns a next job from the queue
    pub async fn next(self: &Arc<Self>) -> ActiveJobPermit<J> {
        let (id, job) = self.pool_queue().await;

        let active = Arc::new(ActiveJob::new(job));

        {
            let mut lock = self.lock();
            lock.active.insert(id, active.clone());
        }

        ActiveJobPermit {
            dispatcher: self.clone(),
            job_id: id,
            inner: active,
            remove_on_drop: true,
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

    /// Removes the [`Job`] with the specified [`JobId`] from the queue
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

    /// Returns the next `Job` from the queue or waits for a new one to be added
    async fn pool_queue(&self) -> (JobId, J) {
        loop {
            let permit = self.on_new_job.notified();

            if let Some(j) = {
                let mut lock = self.lock();
                lock.queue.pop_front()
            } {
                return j;
            }

            permit.await;
        }
    }

    /// Removes the [`ActiveJob`] without triggering cancellation
    fn remove_active(&self, id: JobId) {
        let mut lock = self.lock();
        if let Some(removed) = lock.active.remove(&id)
            && !removed.job().allow_concurrency()
        {
            lock.pending_kinds.remove(removed.job().kind());
        }
    }

    /// Removes the [`ActiveJob`] and requeues it at the end of the queue
    fn requeue(&self, id: JobId) -> bool {
        let mut lock = self.lock();
        let Some(removed) = lock.active.remove(&id) else {
            return false;
        };

        let job = removed.job().clone();

        lock.queue.push_back((id, job));
        drop(lock);

        self.on_new_job.notify_one();

        true
    }
}

impl<J: BackgroundJob> Default for JobsDispatcher<J> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct JobsQueueInner<J: BackgroundJob> {
    queue: VecDeque<(JobId, J)>,
    active: HashMap<JobId, Arc<ActiveJob<J>>>,
    pending_kinds: HashSet<&'static str>,
}

/// Permit for active job
///
/// Upon a Drop, the job is automatically removed from the list of active jobs
pub struct ActiveJobPermit<J: BackgroundJob> {
    dispatcher: Arc<JobsDispatcher<J>>,
    job_id: JobId,
    inner: Arc<ActiveJob<J>>,
    remove_on_drop: bool,
}

impl<J: BackgroundJob> ActiveJobPermit<J> {
    /// Returns a reference to the job of this [`ActiveJobPermit`]
    pub fn inner(&self) -> &ActiveJob<J> {
        &self.inner
    }

    /// Removes the current job from the list of active jobs and returns it to the end of the queue
    pub fn requeue(mut self) {
        self.dispatcher.requeue(self.job_id);
        self.remove_on_drop = false;
    }

    /// Removes the current job from the list of active jobs
    pub fn remove(mut self) {
        self.dispatcher.remove_active(self.job_id);
        self.remove_on_drop = false;
    }
}

impl<J: BackgroundJob> Drop for ActiveJobPermit<J> {
    fn drop(&mut self) {
        if !self.remove_on_drop {
            return;
        }

        self.dispatcher.remove_active(self.job_id);
    }
}

/// Active job currently being processed
#[derive(Debug)]
pub struct ActiveJob<J: BackgroundJob> {
    /// Payload of the active job
    job: J,
    /// Task cancellation token
    cancel: Notify,
}

impl<J: BackgroundJob> ActiveJob<J> {
    /// Creates a new [`ActiveJob`]
    pub fn new(job: J) -> ActiveJob<J> {
        ActiveJob {
            job,
            cancel: Notify::new(),
        }
    }

    /// Sends a signal to cancel job execution to the worker that picked up the job
    pub fn cancel(&self) {
        self.cancel.notify_one();
    }

    /// Waiting for a cancellation signal for the [`ActiveJob`]
    pub async fn cancelled(&self) {
        self.cancel.notified().await
    }

    /// Returns a reference to the `job` of this [`ActiveJob`]
    pub fn job(&self) -> &J {
        &self.job
    }
}
