use std::str::FromStr;

use chrono::Utc;
use db::types::{
    UpdateResult,
    patch::{AssetFeaturesPatch, MediaFilePatch},
};
use mimetype::MimeKind;
use models::{
    assets::{AssetState, view::AssetView},
    media::{MediaFile, MediaVariant},
    types::{AssetId, MediaId},
};
use result::{ErrorKind, Result, error::ResultExt};
use storage::{StoragePath, global::GlobalPathData};
use tokio::time::Instant;

use crate::{
    media::{
        processing::{image::ImageProcessor, video::VideoProcessor},
        store::{store_image_variant, store_video_variant},
    },
    runtime::WorkerContext,
};

/// Processes media of pending assets or assets lacking certain features
pub async fn process_unprocessed_media(ctx: &WorkerContext) -> Result<usize> {
    let mut processed = 0;

    // Retrieves unprocessed assets as long as there are any in the database
    loop {
        // Receives unprocessed assets
        let unprocessed = ctx.db.assets.get_for_processing(50).await?;

        // Triggers processing for all unprocessed assets
        for asset in &unprocessed {
            process_asset_media(ctx, asset).await?;
            processed += 1;
        }

        // If there are no unprocessed assets left, breaks the loop
        if unprocessed.len() < 50 {
            break;
        }
    }
    Ok(processed)
}

/// Calls for processing of an [`Asset`] by id
pub async fn process_asset_by_id(ctx: &WorkerContext, id: AssetId) -> Result<()> {
    // Retrieves an Asset by ID
    let asset = {
        let asset = ctx.db.assets.get_by_id(id).await;
        match asset {
            Ok(a) => a,
            Err(e) if matches!(e.kind(), ErrorKind::NotFound) => return Ok(()),
            Err(e) => return Err(e),
        }
    };

    // Calls processing for the asset
    process_asset_media(ctx, &asset).await
}

pub async fn process_asset_media(ctx: &WorkerContext, asset: &AssetView) -> Result<()> {
    if !asset.inner.need_processing(&asset.features, Utc::now()) {
        return Ok(());
    }

    // Setting the asset status to `processing`
    if change_asset_state(ctx, asset.inner.id, AssetState::Processing)
        .await?
        .no_changes()
    {
        return Ok(());
    }

    // Records the time when processing begin
    let t0 = Instant::now();

    // Executes a processing pipeline suitable for the asset type
    let media_type = asset.inner.media_type;
    tracing::info!(
        "MediaWorker: Processing {} as {}",
        asset.inner.id,
        media_type
    );

    let res = match media_type {
        MimeKind::Image => process_asset_as_image(ctx, asset).await,
        MimeKind::Video => process_asset_as_video(ctx, asset).await,
    };

    if let Err(e) = res {
        // In the event of a processing error,
        // it sets the state to 'failed' and propagates the error
        change_asset_state(ctx, asset.inner.id, AssetState::Failed).await?;
        return Err(e);
    } else {
        change_asset_state(ctx, asset.inner.id, AssetState::Ready).await?;
    };

    tracing::info!(
        id = ?asset.inner.id, elapsed = t0.elapsed().as_millis(), m_type = ?asset.inner.media_type,
        "MediaWorker: Asset media processed"
    );

    Ok(())
}

async fn process_asset_as_image(ctx: &WorkerContext, asset: &AssetView) -> Result<()> {
    // Retrieves information about the original media file
    let original = get_original(ctx, asset.media_id()).await?;

    // Retrieves the original file from the storage
    let path = StoragePath::from_str(&original.storage_path).to_app_err()?;
    let file = ctx.storage.open(&path).await?;

    // Decodes the image from the original file
    let processor = ImageProcessor::decode(file).await?;

    // Checks which variants already exist for the specified asset
    let variants = asset.media_variants();

    // Generates and saves a thumbnail
    if !variants.contains(&MediaVariant::Thumbnail) {
        let thumbnail = processor.generate_thumbnail()?;
        store_image_variant(ctx, thumbnail, asset.media_id()).await?;
    }

    // Retrieves the basic image parameters and features
    let features = processor.extract_features();

    // Writes features to the database
    let patch = features.into();
    ctx.db.assets.update_features(asset.inner.id, patch).await?;

    Ok(())
}

async fn process_asset_as_video(ctx: &WorkerContext, asset: &AssetView) -> Result<()> {
    // Retrieves information about the original media file
    let original = get_original(ctx, asset.media_id()).await?;

    // Copies the video to a temporary directory for processing
    let path = StoragePath::from_str(&original.storage_path).to_app_err()?;
    let original_video = ctx.storage.open_local(&path).await?;

    // Opens the video
    let processor = VideoProcessor::open_video(original_video.path()).await?;

    // Extracts metadata from video
    let meta = processor.metadata();

    // Checks which variants already exist for the specified asset
    let variants = asset.media_variants();

    // Generates thumbnail
    if !variants.contains(&MediaVariant::Thumbnail) {
        let thumbnail = processor.generate_thumbnail().await?;
        store_image_variant(ctx, thumbnail, asset.media_id()).await?;
    }

    // Generates loop preview
    if !variants.contains(&MediaVariant::LoopPreview) {
        let reserve = ctx.storage.reserve(GlobalPathData::new(
            &asset.inner.media_id.to_string(),
            MediaVariant::LoopPreview.as_str(),
        ));

        let loop_preview = processor.generate_loop_preview(reserve).await?;
        store_video_variant(ctx, loop_preview, asset.media_id()).await?;
    }

    // Retrieves the basic image parameters and features
    let features = processor.extract_features().await?;

    // Writes features to the database
    let res = &meta.video.resolution;
    let (width, height) = (res.width, res.height);
    let patch = AssetFeaturesPatch::from(features)
        .width(Some(width))
        .height(Some(height));
    ctx.db.assets.update_features(asset.inner.id, patch).await?;

    let duration = (meta.video.duration_secs * 1000.0).round() as i64;
    let file_patch = MediaFilePatch::new().duration_ms(Some(duration));
    ctx.db.media.update_file(&original.id, file_patch).await?;

    Ok(())
}

async fn change_asset_state(
    ctx: &WorkerContext,
    id: AssetId,
    state: AssetState,
) -> Result<UpdateResult<AssetView>> {
    ctx.db.assets.update_state(id, state).await
}

async fn get_original(ctx: &WorkerContext, id: &MediaId) -> Result<MediaFile> {
    ctx.db.media.get_variant(id, MediaVariant::Original).await
}
