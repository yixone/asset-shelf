use crate::{
    JobId, ScheduledJob,
    job::{BackgroundJob, ExecutionStatus},
};

/// Background jobs snapshot
#[derive(Debug)]
pub struct JobsSnapshot<J: BackgroundJob> {
    pub schedule: JobsSchedulerSnapshot<J>,
    pub queue: JobsQueueSnapshot<J>,
}

/// Background jobs queue snapshot
#[derive(Debug)]
pub struct JobsQueueSnapshot<J: BackgroundJob> {
    pub active: Vec<(JobId, J)>,
    pub queue: Vec<(JobId, J)>,
    pub executed: Vec<(JobId, J, ExecutionStatus)>,
}

/// Background jobs scheduler snapshot
#[derive(Debug)]
pub struct JobsSchedulerSnapshot<J: BackgroundJob> {
    pub next: Option<ScheduledJob<J>>,
    pub queue: Vec<ScheduledJob<J>>,
}
