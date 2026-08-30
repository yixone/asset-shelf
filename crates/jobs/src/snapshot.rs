use crate::{Job, JobId, ScheduledJob};

/// Background jobs snapshot
#[derive(Debug)]
pub struct JobsSnapshot {
    pub schedule: JobsSchedulerSnapshot,
    pub queue: JobsQueueSnapshot,
}

/// Background jobs queue snapshot
#[derive(Debug)]
pub struct JobsQueueSnapshot {
    pub active: Vec<(JobId, Job)>,
    pub queue: Vec<(JobId, Job)>,
}

/// Background jobs scheduler snapshot
#[derive(Debug)]
pub struct JobsSchedulerSnapshot {
    pub next: Option<ScheduledJob>,
    pub queue: Vec<ScheduledJob>,
}
