use std::fmt::Debug;

use flake_id::FlakeId;

/// Background job identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobId(FlakeId);

impl From<FlakeId> for JobId {
    fn from(id: FlakeId) -> Self {
        JobId(id)
    }
}

pub trait BackgroundJob: Clone + Send + Sync + Debug + PartialEq + Eq {
    type Error;
    type Context: Send + Sync;

    /// Returns the `kind` of this [`BackgroundJob`]
    fn kind(&self) -> &'static str;

    /// Returns `true` if concurrent execution is allowed for the current [`BackgroundJob`]
    fn allow_concurrency(&self) -> bool;

    /// Executes the task in a background worker
    fn execute(&self, ctx: &Self::Context) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
