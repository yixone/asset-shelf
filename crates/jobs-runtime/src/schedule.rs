use std::{
    collections::BinaryHeap,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use tokio::{sync::Notify, time::Instant};

use crate::{JobsSchedulerSnapshot, job::BackgroundJob};

/// Jobs Execution Schedule
#[derive(Debug)]
pub struct Schedule<J: BackgroundJob> {
    inner: Mutex<ScheduleInner<J>>,
    notify: Notify,
    id: Mutex<u64>,
}

#[derive(Debug)]
pub struct ScheduleInner<J: BackgroundJob> {
    scheduled_queue: BinaryHeap<ScheduledJob<J>>,
    waiting: Option<ScheduledJob<J>>,
}

impl<J: BackgroundJob> Schedule<J> {
    /// Creates a new [`Schedule`]
    pub fn new() -> Self {
        Schedule {
            inner: Mutex::new(ScheduleInner {
                scheduled_queue: BinaryHeap::new(),
                waiting: None,
            }),
            notify: Notify::new(),
            id: Mutex::new(0),
        }
    }

    fn lock<'a>(&'a self) -> MutexGuard<'a, ScheduleInner<J>> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Returns the next id of this [`Schedule<J>`]
    fn next_id(&self) -> ScheduledJobId {
        let mut lock = self.id.lock().unwrap_or_else(|f| f.into_inner());

        let id = *lock;

        *lock = lock.saturating_add(1);

        ScheduledJobId(id)
    }

    /// Returns the snapshot of this [`Schedule`]
    pub fn snapshot(&self) -> JobsSchedulerSnapshot<J> {
        let lock = self.lock();
        JobsSchedulerSnapshot {
            next: lock.waiting.clone(),
            queue: lock.scheduled_queue.iter().cloned().collect(),
        }
    }

    /// Waits for the time of the next scheduled task and returns it
    pub async fn next_run(&self) -> Option<J> {
        loop {
            let scheduled = self.take_near()?;

            tokio::select! {
                _ = self.notify.notified() => {
                    self.restore_waiting(scheduled);
                    continue
                }
                _ = tokio::time::sleep_until(scheduled.schedule.next_run()) => {
                    let job = self.handle_processed_schedule(scheduled);
                    return Some(job)
                }
            }
        }
    }

    pub fn schedule_many(&self, jobs: impl IntoIterator<Item = (J, JobSchedule)>) {
        let mut lock = self.lock();
        let mut added = 0;

        for (j, s) in jobs {
            lock.scheduled_queue.push(ScheduledJob {
                id: self.next_id(),
                job: j.clone(),
                schedule: s,
            });

            added += 1;
        }

        drop(lock);

        if added > 0 {
            self.notify.notify_last();
        }
    }

    /// Retrieves the nearest job and moves it to current
    fn take_near(&self) -> Option<ScheduledJob<J>> {
        let mut lock = self.lock();
        let scheduled = lock.scheduled_queue.pop()?;
        lock.waiting = Some(scheduled.clone());

        Some(scheduled)
    }

    /// Returns the waiting job to the schedule
    fn restore_waiting(&self, waiting: ScheduledJob<J>) {
        let mut lock = self.lock();

        if let Some(scheduled) = &lock.waiting
            && scheduled.id == waiting.id
        {
            lock.scheduled_queue.push(waiting);
            lock.waiting = None;
        }
    }

    fn handle_processed_schedule(&self, scheduled: ScheduledJob<J>) -> J {
        let mut lock = self.lock();

        if let JobSchedule::Interval { interval, .. } = scheduled.schedule {
            let rescheduled = ScheduledJob {
                id: scheduled.id,
                job: scheduled.job.clone(),
                schedule: scheduled.schedule.move_next_run(interval),
            };

            lock.scheduled_queue.push(rescheduled);
        }

        lock.waiting = None;

        scheduled.job
    }
}

/// Scheduled background job
#[derive(Debug, Clone)]
pub struct ScheduledJob<J: BackgroundJob> {
    pub(crate) id: ScheduledJobId,
    pub(crate) job: J,
    pub(crate) schedule: JobSchedule,
}

impl<J: BackgroundJob> ScheduledJob<J> {
    /// Returns a reference to the job of this [`ScheduledJob`]
    pub fn job(&self) -> &J {
        &self.job
    }

    /// Returns the schedule of this [`ScheduledJob`]
    pub fn schedule(&self) -> JobSchedule {
        self.schedule
    }
}

impl<J: BackgroundJob> Ord for ScheduledJob<J> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.schedule.next_run().cmp(&self.schedule.next_run())
    }
}

impl<J: BackgroundJob> PartialOrd for ScheduledJob<J> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<J: BackgroundJob> PartialEq for ScheduledJob<J> {
    fn eq(&self, other: &Self) -> bool {
        self.job == other.job
    }
}

impl<J: BackgroundJob> Eq for ScheduledJob<J> {}

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

    /// Shifts the scheduled job next run time
    pub fn move_next_run(self, move_on: Duration) -> Self {
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

/// Scheduler job identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScheduledJobId(u64);
