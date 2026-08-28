use std::{collections::BinaryHeap, sync::Arc, time::Duration};

use tokio::{task::JoinHandle, time::Instant};
use tokio_util::sync::CancellationToken;

use crate::{Job, resolver::JobsResolver};

/// Background Job Scheduler
///
/// Periodically adds background tasks to the queue
#[derive(Debug)]
pub struct JobsScheduler {
    scheduled: BinaryHeap<ScheduledJob>,
    resolver: Arc<JobsResolver>,
}

impl JobsScheduler {
    /// Creates a new [`JobsScheduler`]
    pub fn new(resolver: Arc<JobsResolver>) -> Self {
        Self {
            scheduled: BinaryHeap::new(),
            resolver,
        }
    }

    pub fn schedule(mut self, job: Job, schedule: JobSchedule) -> Self {
        self.scheduled.push(ScheduledJob { job, schedule });
        self
    }

    async fn next_run(&mut self) -> Option<Job> {
        let scheduled = self.scheduled.pop()?;

        tokio::time::sleep_until(*scheduled.schedule.next_run()).await;

        if let JobSchedule::Interval { interval, .. } = scheduled.schedule {
            let rescheduled = ScheduledJob {
                job: scheduled.job.clone(),
                schedule: scheduled.schedule.move_run(interval),
            };
            self.scheduled.push(rescheduled);
        }

        Some(scheduled.job)
    }

    /// Starts the loop for this [`JobsScheduler`]
    pub fn run(mut self, cancel: CancellationToken) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(job) = self.next_run() => {
                        self.resolver.queue(job).await;
                    }
                    _ = cancel.cancelled() => {
                        return;
                    }
                }
            }
        })
    }
}

/// Scheduled background job
#[derive(Debug)]
pub struct ScheduledJob {
    pub(crate) job: Job,
    pub(crate) schedule: JobSchedule,
}

impl Ord for ScheduledJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.schedule.next_run().cmp(self.schedule.next_run())
    }
}

impl PartialOrd for ScheduledJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledJob {
    fn eq(&self, other: &Self) -> bool {
        self.job == other.job
    }
}

impl Eq for ScheduledJob {}

/// Job execution schedule
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobSchedule {
    Interval {
        interval: Duration,
        next_run: Instant,
    },
    Once {
        run: Instant,
    },
}

impl JobSchedule {
    pub fn interval(interval: Duration) -> Self {
        let next_run = Instant::now() + interval;
        Self::Interval { interval, next_run }
    }

    pub fn once(execute_in: Duration) -> Self {
        let run = Instant::now() + execute_in;
        Self::Once { run }
    }

    pub fn is_once(&self) -> bool {
        matches!(self, JobSchedule::Once { .. })
    }

    pub fn is_interval(&self) -> bool {
        matches!(self, JobSchedule::Interval { .. })
    }

    pub fn move_run(self, move_on: Duration) -> Self {
        match self {
            JobSchedule::Interval { interval, next_run } => JobSchedule::Interval {
                interval,
                next_run: next_run + move_on,
            },
            JobSchedule::Once { run } => JobSchedule::Once { run: run + move_on },
        }
    }

    pub fn next_run(&self) -> &Instant {
        match self {
            JobSchedule::Interval { next_run, .. } => next_run,
            JobSchedule::Once { run } => run,
        }
    }
}
