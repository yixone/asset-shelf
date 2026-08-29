use std::{collections::BinaryHeap, time::Duration};

use tokio::{sync::Mutex, time::Instant};

use crate::Job;

/// Jobs Execution Schedule
#[derive(Debug)]
pub struct Schedule {
    inner: Mutex<BinaryHeap<ScheduledJob>>,
    current: Mutex<Option<ScheduledJob>>,
}

impl Schedule {
    /// Creates a new [`Schedule`]
    pub fn new(jobs: &[(Job, JobSchedule)]) -> Self {
        let mut schedule = BinaryHeap::with_capacity(jobs.len());
        for (j, s) in jobs {
            schedule.push(ScheduledJob {
                job: j.clone(),
                schedule: *s,
            });
        }

        Schedule {
            inner: Mutex::new(schedule),
            current: Mutex::new(None),
        }
    }

    /// Waits for the time of the next scheduled task and returns it
    pub async fn next_run(&self) -> Option<Job> {
        let scheduled = {
            let mut lock = self.inner.lock().await;
            lock.pop()
        }?;

        {
            let mut lock = self.current.lock().await;
            *lock = Some(scheduled.clone());
        }

        tokio::time::sleep_until(*scheduled.schedule.next_run()).await;

        if let JobSchedule::Interval { interval, .. } = scheduled.schedule {
            let rescheduled = ScheduledJob {
                job: scheduled.job.clone(),
                schedule: scheduled.schedule.move_run(interval),
            };
            let mut lock = self.inner.lock().await;
            lock.push(rescheduled);
        }

        {
            let mut lock = self.current.lock().await;
            *lock = None;
        }

        Some(scheduled.job)
    }
}

/// Scheduled background job
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
