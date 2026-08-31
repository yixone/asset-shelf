use std::fmt::Debug;

use flake_id::FlakeId;

/// Job to be executed by a background worker
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Job {
    ProcessAssetMedia { id: models::types::AssetId },
    ProcessUnprocessedAssets,
    CleanupStorageMedia,
    RemoveMediaAfterAssetDeletion { id: models::types::MediaId },
}

impl Job {
    /// Returns the `kind` of this [`Job`]
    pub fn kind(&self) -> &'static str {
        match self {
            Job::ProcessAssetMedia { .. } => "process_asset_media",
            Job::ProcessUnprocessedAssets => "process_uprocessed_assets",

            Job::CleanupStorageMedia => "cleanup_storage_media",
            Job::RemoveMediaAfterAssetDeletion { .. } => "remove_media_after_asset_creation",
        }
    }

    /// Returns `true` if concurrent execution is allowed for the current [`Job`]
    pub fn allow_concurrency(&self) -> bool {
        !matches!(self, Job::CleanupStorageMedia)
    }
}

/// Background job identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobId(FlakeId);

impl From<FlakeId> for JobId {
    fn from(id: FlakeId) -> Self {
        JobId(id)
    }
}
