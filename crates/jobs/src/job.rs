use std::sync::Arc;

use flake_id::FlakeId;
use tokio::sync::Notify;

/// Job to be executed by a background worker
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Job {
    ProcessAssetMedia { id: models::types::AssetId },
    CleanupStorageMedia,
}

impl Job {
    /// Returns the `kind` of this [`Job`]
    pub fn kind(&self) -> &'static str {
        match self {
            Job::ProcessAssetMedia { .. } => "process_asset_media",
            Job::CleanupStorageMedia => "cleanup_storage_media",
        }
    }

    /// Returns `true` if concurrent execution is allowed for the current [`Job`]
    pub fn allow_concurrency(&self) -> bool {
        match self {
            Job::ProcessAssetMedia { .. } => true,
            Job::CleanupStorageMedia => false,
        }
    }
}

/// Active job currently being processed
#[derive(Debug)]
pub struct ActiveJob {
    /// Payload of the active job
    job: Job,
    /// Task cancellation token
    cancel: Notify,
}

impl ActiveJob {
    /// Creates a new [`ActiveJob`]
    pub fn new(job: Job) -> ActiveJob {
        ActiveJob {
            job,
            cancel: Notify::new(),
        }
    }

    /// Sends a signal to cancel job execution to the worker that picked up the job
    pub fn cancel(self: Arc<Self>) {
        self.cancel.notify_one();
    }

    /// Returns a reference to the job of this [`ActiveJob`]
    pub fn job(&self) -> &Job {
        &self.job
    }
}

/// Scheduler job identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(FlakeId);

impl From<FlakeId> for JobId {
    fn from(id: FlakeId) -> Self {
        JobId(id)
    }
}
