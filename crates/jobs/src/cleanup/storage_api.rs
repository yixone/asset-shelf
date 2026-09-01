use std::str::FromStr;

use models::{assets::Asset, media::view::MediaView};
use result::{Result, error::ResultExt};
use storage::StoragePath;

use crate::JobContext;

/// Deletes media and all associated files
pub async fn delete_media(ctx: &JobContext, media: MediaView) -> Result<()> {
    for f in &media.files {
        ctx.db.media.delete_file(&f.id).await?;

        let path = StoragePath::from_str(&f.storage_path).to_app_err()?;
        if ctx.storage.remove_safely(&path).await {
            tracing::info!(path = ?f.storage_path, "CleanupWorker: Removed storage media file");
        }
    }

    ctx.db.media.delete(&media.inner.id).await?;

    Ok(())
}

/// Removes an asset
pub async fn delete_asset(ctx: &JobContext, asset: &Asset) -> Result<()> {
    ctx.db.assets.delete(asset.id).await?;
    Ok(())
}
