use chrono::Utc;
use models::{
    media::{MediaFile, MediaVariant},
    types::MediaId,
};
use result::Result;
use storage::global::GlobalPathData;

use crate::{
    media::processing::{image::GeneratedImageVariant, video::GeneratedVideoVariant},
    runtime::WorkerContext,
};

pub async fn store_image_variant(
    ctx: &WorkerContext,
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
    ctx: &WorkerContext,
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
