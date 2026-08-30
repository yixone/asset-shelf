use std::{
    collections::BinaryHeap,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use tokio::time::Instant;

use crate::{Job, JobsSchedulerSnapshot};

/// Jobs Execution Schedule
#[derive(Debug)]
pub struct Schedule {
    inner: Mutex<ScheduleInner>,
}

#[derive(Debug)]
pub struct ScheduleInner {
    scheduled_queue: BinaryHeap<ScheduledJob>,
    waiting: Option<ScheduledJob>,
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
            inner: Mutex::new(ScheduleInner {
                scheduled_queue: schedule,
                waiting: None,
            }),
        }
    }

    fn lock<'a>(&'a self) -> MutexGuard<'a, ScheduleInner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Returns the snapshot of this [`Schedule`]
    pub fn snapshot(&self) -> JobsSchedulerSnapshot {
        let lock = self.lock();
        JobsSchedulerSnapshot {
            next: lock.waiting.clone(),
            queue: lock.scheduled_queue.iter().cloned().collect(),
        }
    }

    /// Waits for the time of the next scheduled task and returns it
    pub async fn next_run(&self) -> Option<Job> {
        let scheduled = self.take_near()?;

        tokio::time::sleep_until(scheduled.schedule.next_run()).await;

        let job = self.handle_processed_schedule(scheduled).await;
        Some(job)
    }

    /// Retrieves the nearest job and moves it to current
    fn take_near(&self) -> Option<ScheduledJob> {
        let mut lock = self.lock();
        let scheduled = lock.scheduled_queue.pop()?;
        lock.waiting = Some(scheduled.clone());

        Some(scheduled)
    }

    async fn handle_processed_schedule(&self, scheduled: ScheduledJob) -> Job {
        let mut lock = self.lock();

        if let JobSchedule::Interval { interval, .. } = scheduled.schedule {
            let rescheduled = ScheduledJob {
                job: scheduled.job.clone(),
                schedule: scheduled.schedule.move_run(interval),
            };

            lock.scheduled_queue.push(rescheduled);
        }

        lock.waiting = None;

        scheduled.job
    }
}

/// Scheduled background job
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub(crate) job: Job,
    pub(crate) schedule: JobSchedule,
}

impl ScheduledJob {
    /// Returns a reference to the job of this [`ScheduledJob`]
    pub fn job(&self) -> &Job {
        &self.job
    }

    /// Returns the schedule of this [`ScheduledJob`]
    pub fn schedule(&self) -> JobSchedule {
        self.schedule
    }
}

impl Ord for ScheduledJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.schedule.next_run().cmp(&self.schedule.next_run())
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
    /// Creates a new interval [`JobSchedule`]
    pub fn interval(interval: Duration) -> Self {
        let next_run = Instant::now() + interval;
        Self::Interval { interval, next_run }
    }

    /// Creates a new one-time [`JobSchedule`]
    pub fn once(execute_in: Duration) -> Self {
        let run = Instant::now() + execute_in;
        Self::Once { run }
    }

    /// Returns `true` if the [`JobSchedule`] is a [`JobSchedule::Once`]
    pub fn is_once(&self) -> bool {
        matches!(self, JobSchedule::Once { .. })
    }

    /// Returns `true` if the [`JobSchedule`] is a [`JobSchedule::Interval`]
    pub fn is_interval(&self) -> bool {
        matches!(self, JobSchedule::Interval { .. })
    }

    /// Shifts the scheduled execution time
    pub fn move_run(self, move_on: Duration) -> Self {
        match self {
            JobSchedule::Interval { interval, next_run } => JobSchedule::Interval {
                interval,
                next_run: next_run + move_on,
            },
            JobSchedule::Once { run } => JobSchedule::Once { run: run + move_on },
        }
    }

    /// Returns the scheduled execution time of the job
    pub fn next_run(&self) -> Instant {
        match self {
            JobSchedule::Interval { next_run, .. } => *next_run,
            JobSchedule::Once { run } => *run,
        }
    }

    /// Returns the scheduled execution time of the job in [`chrono::Utc`]
    pub fn scheduled_time_utc(&self) -> chrono::DateTime<chrono::Utc> {
        let delta = self.next_run().saturating_duration_since(Instant::now());
        chrono::Utc::now() + delta
    }
}
