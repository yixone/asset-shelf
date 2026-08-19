use chrono::Utc;
use db::types::Pagination;
use models::types::AssetsOrdering;
use result::Result;

use crate::{cleanup::delete, runtime::WorkerContext};

const BATCH_LIMIT: u32 = 50;
const BATCH: Pagination = Pagination::new(BATCH_LIMIT, 0);

pub async fn cleanup_orphaned(ctx: &WorkerContext) -> Result<usize> {
    let mut deleted = 0;

    loop {
        let media = ctx.db.media.get_orphans(BATCH_LIMIT).await?;

        let count = media.len();

        for m in media {
            delete::delete_media(ctx, m).await?;
            deleted += 1;
        }

        if count < 50 {
            break;
        }
    }

    Ok(deleted)
}

pub async fn cleanup_deleted_assets(ctx: &WorkerContext) -> Result<usize> {
    let mut deleted = 0;
    let now = Utc::now();
    let retention_time = chrono::Duration::days(30);

    loop {
        let marked = ctx
            .db
            .assets
            .get_deleted(BATCH, AssetsOrdering::Oldest)
            .await?;

        let mut processed = 0;

        for a in &marked {
            let deleted = a
                .inner
                .deleted_at
                .expect("asset_repo.get_deleted_list(..) returned asset without deleted_at");

            if (now - deleted) >= retention_time {
                delete::delete_asset(ctx, &a.inner).await?;
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
