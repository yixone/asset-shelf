// !FIXME: Extract the cleanup code from the `jobs` module

use std::str::FromStr;

use chrono::Utc;
use db::types::Pagination;
use models::{
    assets::Asset,
    media::{MediaFile, view::MediaView},
    types::{AssetsOrdering, MediaId},
};
use result::{Result, error::ResultExt};
use storage::StoragePath;

use crate::JobContext;

const BATCH_LIMIT: u32 = 50;
const BATCH: Pagination = Pagination::new(BATCH_LIMIT, 0);

pub async fn cleanup_orphaned(ctx: &JobContext) -> Result<usize> {
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

pub async fn cleanup_deleted_assets(ctx: &JobContext) -> Result<usize> {
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

pub async fn remove_media_by_id(ctx: &JobContext, id: &MediaId) -> Result<()> {
    let media = ctx.db.media.get_by_id(id).await?;
    delete::delete_media(ctx, media).await
}

mod delete {
    use super::*;

    pub async fn delete_media(ctx: &JobContext, media: MediaView) -> Result<()> {
        for f in &media.files {
            ctx.db.media.delete_file(&f.id).await?;
            delete_storage_file(ctx, f).await?;
        }

        ctx.db.media.delete(&media.inner.id).await?;

        Ok(())
    }

    pub async fn delete_asset(ctx: &JobContext, asset: &Asset) -> Result<()> {
        ctx.db.assets.delete(asset.id).await?;
        Ok(())
    }

    pub async fn delete_storage_file(ctx: &JobContext, file: &MediaFile) -> Result<()> {
        let path = StoragePath::from_str(&file.storage_path).to_app_err()?;
        if ctx.storage.remove_safely(&path).await {
            tracing::info!(path = ?file.storage_path, "CleanupWorker: Removed storage media file");
        }
        Ok(())
    }
}
