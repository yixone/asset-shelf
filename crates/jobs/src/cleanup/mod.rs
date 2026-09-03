use chrono::Utc;
use db::types::Pagination;
use models::types::{AssetsOrdering, MediaId};
use result::Result;

use crate::JobContext;

mod storage_api;

const BULK_BATCH: Pagination = Pagination::new(50, 0);

/// Deletes all media that no one is using
pub async fn cleanup_orphaned(ctx: &JobContext) -> Result<usize> {
    let mut deleted = 0;

    loop {
        let media = ctx.db.media.get_orphans(BULK_BATCH.limit()).await?;

        let count = media.len();

        for m in media {
            storage_api::delete_media(ctx, m).await?;
            deleted += 1;
        }

        if count < 50 {
            break;
        }
    }

    Ok(deleted)
}

/// Deletes assets marked as deleted with an expired retention period
pub async fn cleanup_deleted_assets(ctx: &JobContext) -> Result<usize> {
    let mut deleted = 0;
    let now = Utc::now();

    loop {
        let marked = ctx
            .db
            .assets
            .get_deleted(BULK_BATCH, AssetsOrdering::Oldest)
            .await?;

        let mut processed = 0;

        for a in &marked {
            if a.inner.need_cleanup(now) {
                storage_api::delete_asset(ctx, &a.inner).await?;
                processed += 1;
            } else {
                tracing::info!("Reached an asset that was deleted less than 30 days ago");
                break;
            }
        }

        deleted += processed;
        if processed != marked.len() || marked.len() < 50 {
            break;
        }
    }

    Ok(deleted)
}

/// Deletes media by ID
pub async fn remove_media_by_id(ctx: &JobContext, id: &MediaId) -> Result<()> {
    let media = ctx.db.media.get_by_id(id).await?;
    storage_api::delete_media(ctx, media).await
}
