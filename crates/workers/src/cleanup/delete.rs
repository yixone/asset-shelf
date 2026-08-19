use std::str::FromStr;

use models::{
    assets::Asset,
    media::{MediaFile, view::MediaView},
};
use result::{Result, error::ResultExt};
use storage::StoragePath;

use crate::runtime::WorkerContext;

pub async fn delete_media(ctx: &WorkerContext, media: MediaView) -> Result<()> {
    for f in &media.files {
        ctx.db.media.delete_file(&f.id).await?;
        delete_storage_file(ctx, f).await?;
    }

    ctx.db.media.delete(&media.inner.id).await?;

    Ok(())
}

pub async fn delete_asset(ctx: &WorkerContext, asset: &Asset) -> Result<()> {
    ctx.db.assets.delete(asset.id).await?;
    Ok(())
}

pub async fn delete_storage_file(ctx: &WorkerContext, file: &MediaFile) -> Result<()> {
    let path = StoragePath::from_str(&file.storage_path).to_app_err()?;
    if ctx.storage.remove_safely(&path).await {
        tracing::info!(path = ?file.storage_path, "CleanupWorker: Removed storage media file");
    }
    Ok(())
}
