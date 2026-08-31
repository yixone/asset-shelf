use result::Result;

use crate::{Job, WorkerContext};

mod storage_cleanup;

mod asset_processing;

pub async fn handle_job(job: &Job, ctx: &WorkerContext) -> Result<()> {
    match job {
        Job::ProcessAssetMedia { id } => asset_processing::process_asset_by_id(ctx, *id).await,
        Job::ProcessUnprocessedAssets => {
            let processed = asset_processing::process_unprocessed_media(ctx).await?;

            tracing::info!(
                processed,
                "Background processing of pending media is complete;"
            );

            Ok(())
        }

        Job::CleanupStorageMedia => {
            let mut removed = 0;

            removed += storage_cleanup::cleanup_orphaned(ctx).await?;
            removed += storage_cleanup::cleanup_deleted_assets(ctx).await?;

            tracing::info!(removed, "Background storage cleanup completed;");

            Ok(())
        }
        Job::RemoveMediaAfterAssetDeletion { id } => {
            storage_cleanup::remove_media_by_id(ctx, id).await
        }
    }
}
