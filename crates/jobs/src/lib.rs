use std::sync::Arc;

use db::RepositoryContext;
use flake_id::FlakeIdGenerator;
use jobs_runtime::BackgroundJob;
use storage::Storage;

mod cleanup;
mod process;

/// Job to be executed by a background worker
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Job {
    /// Media processing job for the asset with the specified ID
    ProcessAssetMedia { id: models::types::AssetId },
    /// The job is to process assets that are not considered processed
    ProcessUnprocessedAssets,

    /// Media storage cleanup job
    CleanupStorageMedia,
    /// Job of removing media files for a removed asset
    RemoveMediaAfterAssetDeletion { id: models::types::MediaId },
}

/// Shared context of the jobs
#[derive(Clone)]
pub struct JobContext {
    pub db: Arc<RepositoryContext>,
    pub flake: Arc<FlakeIdGenerator>,
    pub storage: Arc<Storage>,
}

impl BackgroundJob for Job {
    type Error = result::Error;

    type Context = JobContext;

    fn kind(&self) -> &'static str {
        match self {
            Job::ProcessAssetMedia { .. } => "process_asset_media",
            Job::ProcessUnprocessedAssets => "process_uprocessed_assets",
            Job::CleanupStorageMedia => "cleanup_storage_media",
            Job::RemoveMediaAfterAssetDeletion { .. } => "remove_media_after_asset_creation",
        }
    }

    fn allow_concurrency(&self) -> bool {
        !matches!(self, Job::CleanupStorageMedia)
    }

    async fn execute(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        match self {
            Job::ProcessAssetMedia { id } => process::process_asset_by_id(ctx, *id).await,
            Job::ProcessUnprocessedAssets => {
                let processed = process::process_unprocessed_media(ctx).await?;

                tracing::info!(
                    processed,
                    "Background processing of pending media is complete;"
                );

                Ok(())
            }
            Job::CleanupStorageMedia => {
                let mut removed = 0;

                removed += cleanup::cleanup_orphaned(ctx).await?;
                removed += cleanup::cleanup_deleted_assets(ctx).await?;

                tracing::info!(removed, "Background storage cleanup completed;");

                Ok(())
            }
            Job::RemoveMediaAfterAssetDeletion { id } => cleanup::remove_media_by_id(ctx, id).await,
        }
    }
}
