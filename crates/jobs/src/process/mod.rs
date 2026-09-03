// !FIXME: Extract the processing code from the `jobs` module

use std::time::Instant;

use chrono::Utc;
use db::types::patch::AssetFeaturesPatch;
use mimetype::MimeKind;
use models::assets::AssetState;
use models::assets::view::AssetView;
use models::types::AssetId;
use models::types::Color;
use result::Result;

use crate::JobContext;
use crate::process::image::process_asset_as_image;
use crate::process::storage_api::change_asset_state;
use crate::process::video::process_asset_as_video;

mod image;
mod storage_api;
mod video;

/// Processes media of pending assets or assets lacking certain features
pub async fn process_unprocessed_media(ctx: &JobContext) -> Result<usize> {
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
pub async fn process_asset_by_id(ctx: &JobContext, id: AssetId) -> Result<()> {
    // Retrieves an Asset by ID
    let asset = {
        let asset = ctx.db.assets.get_by_id(id).await;
        match asset {
            Ok(a) => a,
            Err(e) if e.is_not_found() => return Ok(()),
            Err(e) => return Err(e),
        }
    };

    // Calls processing for the asset
    process_asset_media(ctx, &asset).await
}

async fn process_asset_media(ctx: &JobContext, asset: &AssetView) -> Result<()> {
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
        // In the event of a processing error, it sets the state to 'failed' and propagates the error
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

pub struct ExtractedFeatures {
    pub a_hash: i64,
    pub p_hash: i64,
    pub color: Color,
    pub width: u32,
    pub height: u32,
}

impl From<ExtractedFeatures> for AssetFeaturesPatch {
    fn from(f: ExtractedFeatures) -> Self {
        AssetFeaturesPatch::new()
            .a_hash(Some(f.a_hash))
            .p_hash(Some(f.p_hash))
            .height(Some(f.height))
            .width(Some(f.width))
            .accent_color(Some(f.color))
    }
}
