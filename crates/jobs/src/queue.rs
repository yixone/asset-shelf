use std::collections::VecDeque;

use tokio::sync::{Mutex, Notify};

use crate::{Job, job::JobId};

#[derive(Debug)]
pub struct JobsQueue {
    inner: Mutex<VecDeque<(JobId, Job)>>,
    notify: Notify,
}

impl JobsQueue {
    /// Creates a new [`JobsQueue`]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        }
    }

    pub async fn queue(&self, job: (JobId, Job)) {
        {
            let mut lock = self.inner.lock().await;
            lock.push_back(job);
        }
        self.notify.notify_one();
    }

    pub async fn pop(&self) -> (JobId, Job) {
        if let Some(j) = self.try_pop().await {
            return j;
        }

        loop {
            let permit = self.notify.notified();

            if let Some(j) = self.try_pop().await {
                return j;
            }

            permit.await;
        }
    }

    pub async fn drain(&self) -> usize {
        let mut lock = self.inner.lock().await;
        lock.drain(0..).len()
    }

    pub async fn try_pop(&self) -> Option<(JobId, Job)> {
        let mut lock = self.inner.lock().await;
        lock.pop_front()
    }

    pub async fn snapshot(&self) -> Vec<(JobId, &'static str)> {
        let lock = self.inner.lock().await;
        lock.iter().map(|(id, job)| (*id, job.kind())).collect()
    }
}
