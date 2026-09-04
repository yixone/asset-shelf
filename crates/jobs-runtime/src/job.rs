use std::fmt::Debug;

use tokio::sync::Notify;

/// Background job identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobId(pub(crate) u64);

/// Job that can be performed in the background
pub trait BackgroundJob: Clone + Send + Sync + Debug + PartialEq + Eq {
    /// The error returned by the Job
    type Error: Debug;

    /// The context that the Job accepts for DI
    type Context: Send + Sync + 'static;

    /// If `true`, the job can be retried after an error
    fn can_retry(_: &Self::Error) -> bool {
        true
    }

    /// If `true`, the worker will enter a cooldown period after the error
    fn need_cooldown(_: &Self::Error) -> bool {
        false
    }

    /// Returns the `kind` of this [`BackgroundJob`]
    fn kind(&self) -> &'static str;

    /// Returns `true` if concurrent execution is allowed for the current [`BackgroundJob`]
    fn allow_concurrency(&self) -> bool;

    /// Executes the task in a background worker
    fn execute(&self, ctx: &Self::Context) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// Active job currently being processed
#[derive(Debug)]
pub struct ActiveJob<J: BackgroundJob> {
    /// Payload of the active job
    pub(crate) job: J,

    /// Task cancellation token
    pub(crate) cancel: Notify,
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

/// Status of the executed job
#[derive(Debug, Clone, Copy)]
pub enum ExecutionStatus {
    /// The job was completed successfully.
    Success,
    /// Job execution failed with an error
    Failed,
    /// The status of the completed job is undefined
    Undefined,
}
