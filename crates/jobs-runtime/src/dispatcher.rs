use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex, MutexGuard},
};

use tokio::sync::Notify;

use crate::{
    JobsQueueSnapshot,
    job::{ActiveJob, BackgroundJob, ExecutionStatus, JobId},
};

/// Background jobs dispatcher
///
/// Stores and automatically distributes background jobs among workers
#[derive(Debug)]
pub struct JobsDispatcher<J: BackgroundJob> {
    /// Dispatcher internal state
    state: Mutex<JobsDispatcherState<J>>,
    /// Notification of a new job addition
    notify: Notify,
    /// ID counter
    id: Mutex<u64>,
}

impl<J: BackgroundJob> JobsDispatcher<J> {
    /// Creates a new [`JobsDispatcher`]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(JobsDispatcherState {
                queue: VecDeque::new(),
                active: HashMap::new(),
                executed: Vec::new(),
                pending_kinds: HashSet::new(),
            }),
            notify: Notify::new(),
            id: Mutex::new(0),
        }
    }

    /// Returns the mutex guard for the jobs queue contents
    fn lock<'a>(&'a self) -> MutexGuard<'a, JobsDispatcherState<J>> {
        self.state.lock().unwrap_or_else(|f| f.into_inner())
    }

    /// Returns the next id of this [`JobsDispatcher<J>`]
    fn next_id(&self) -> JobId {
        let mut lock = self.id.lock().unwrap_or_else(|f| f.into_inner());

        let id = *lock;
        *lock = lock.saturating_add(1);

        JobId(id)
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
            executed: lock.executed.clone(),
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

            let id = self.next_id();
            lock.queue.push_back((id, job));

            added.push(id);
        }

        drop(lock);

        match added.len() {
            0 => (),
            1 => self.notify.notify_one(),
            _ => self.notify.notify_waiters(),
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

    /// Removes the `Job` with the specified [`JobId`] from the queue
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

    /// Marks the [`ActiveJob`] as executed
    fn mark_executed(&self, id: JobId, status: Option<ExecutionStatus>) {
        let mut lock = self.lock();
        if let Some(removed) = lock.active.remove(&id) {
            let kind = removed.job().kind();

            lock.executed.push((
                id,
                removed.job.clone(),
                status.unwrap_or(ExecutionStatus::Undefined),
            ));

            if !removed.job().allow_concurrency() {
                lock.pending_kinds.remove(kind);
            }
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

        self.notify.notify_one();

        true
    }
}

impl<J: BackgroundJob> Default for JobsDispatcher<J> {
    fn default() -> Self {
        Self::new()
    }
}

/// Jobs dispatcher internal state
#[derive(Debug)]
struct JobsDispatcherState<J: BackgroundJob> {
    /// Queue of pending jobs
    queue: VecDeque<(JobId, J)>,
    /// Active jobs currently being processed
    active: HashMap<JobId, Arc<ActiveJob<J>>>,
    /// Executed jobs
    executed: Vec<(JobId, J, ExecutionStatus)>,
    /// Jobs kinds used to limit the concurrency of certain jobs
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

    /// Removes the current job from the list of active jobs and marks it as executed
    pub fn mark_executed(mut self, status: ExecutionStatus) {
        self.dispatcher.mark_executed(self.job_id, Some(status));
        tracing::info!("Marked as executed: {status:?}");
        self.remove_on_drop = false;
    }
}

impl<J: BackgroundJob> Drop for ActiveJobPermit<J> {
    fn drop(&mut self) {
        if !self.remove_on_drop {
            return;
        }

        self.dispatcher.mark_executed(self.job_id, None);
    }
}
