use chrono::Utc;
use db::types::UpdateResult;
use models::{
    assets::{AssetState, view::AssetView},
    media::{MediaFile, MediaVariant},
    types::{AssetId, MediaId},
};
use result::Result;
use storage::global::GlobalPathData;

use crate::{
    JobContext,
    process::{image::GeneratedImageVariant, video::GeneratedVideoVariant},
};

pub async fn store_image_variant(
    ctx: &JobContext,
    variant: GeneratedImageVariant,
    media_group_id: &MediaId,
) -> Result<()> {
    let file = ctx
        .storage
        .upload(
            GlobalPathData::new(&media_group_id.to_string(), variant.mimetype.as_str()),
            variant.reader,
            |_| Ok(()),
        )
        .await?;

    let media_file = MediaFile {
        id: ctx.flake.get_id_as(),
        media_id: media_group_id.clone(),
        variant: MediaVariant::Thumbnail,
        storage_path: file.global_path().to_string(),
        created_at: Utc::now(),
        size_bytes: file.size_bytes as i64,
        mimetype: variant.mimetype,
        duration_ms: None,
    };

    file.commit().await?;
    ctx.db.media.insert_file(&media_file).await?;

    tracing::info!(
        "MediaWorker: {} generated and saved for media: {}",
        variant.variant,
        media_group_id
    );
    Ok(())
}

pub async fn store_video_variant<'a>(
    ctx: &JobContext,
    variant: GeneratedVideoVariant<'a>,
    media_group_id: &MediaId,
) -> Result<()> {
    let file = variant.reserve.publish().await?;

    let variant_media_file = MediaFile {
        id: ctx.flake.get_id_as(),
        media_id: media_group_id.clone(),
        variant: MediaVariant::LoopPreview,
        storage_path: file.path.to_string(),
        created_at: Utc::now(),
        size_bytes: file.size_bytes as i64,
        mimetype: variant.mimetype,
        duration_ms: Some(variant.duration_milis as i64),
    };

    ctx.db.media.insert_file(&variant_media_file).await?;
    tracing::info!(
        "MediaWorker: {} generated and saved for media: {}",
        variant.variant,
        media_group_id
    );
    Ok(())
}

pub async fn change_asset_state(
    ctx: &JobContext,
    id: AssetId,
    state: AssetState,
) -> Result<UpdateResult<AssetView>> {
    ctx.db.assets.update_state(id, state).await
}

pub async fn get_original(ctx: &JobContext, id: &MediaId) -> Result<MediaFile> {
    ctx.db.media.get_variant(id, MediaVariant::Original).await
}
