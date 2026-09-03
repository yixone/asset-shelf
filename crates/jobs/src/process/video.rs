use std::{path::Path, str::FromStr, time::Duration};

use db::types::patch::{AssetFeaturesPatch, MediaFilePatch};
use media::{
    image::Image,
    video::{
        self, ExtractVideoFragmentParams, FragmentParams, ResizeParams,
        input::MediaInput,
        types::{AudioMode, VideoMetadata},
    },
};
use mimetype::MimeType;
use models::{assets::view::AssetView, media::MediaVariant};
use result::{Result, create_error, error::ResultExt};
use storage::{StoragePath, files::ReservedFile, global::GlobalPathData};
use tokio::time::timeout;

use crate::{
    JobContext,
    process::{
        ExtractedFeatures,
        image::{GeneratedImageVariant, ImageProcessor},
        storage_api::{get_original, store_image_variant, store_video_variant},
    },
};

const PROBE_TIMEOUT: Duration = Duration::from_mins(10);
const EXTRACT_FRAME_TIMEOUT: Duration = Duration::from_mins(2);
const TRANSCODINIG_TIMEOUT: Duration = Duration::from_mins(5);

const LOOP_PREVIEW_FRAGMENT_DURATION_MS: u64 = 5000;
const LOOP_PREVIEW_WIDTH: u32 = 600;

pub struct GeneratedVideoVariant<'a> {
    pub variant: MediaVariant,
    pub mimetype: MimeType,
    pub duration_milis: u64,
    pub reserve: ReservedFile<'a>,
}

pub struct VideoProcessor {
    video: MediaInput,
    meta: VideoMetadata,
}

impl VideoProcessor {
    pub async fn open_video(path: &Path) -> Result<Self> {
        let input = MediaInput::try_new(path).to_app_err()?;

        let metadata = match timeout(PROBE_TIMEOUT, media::video::probe_video(&input)).await {
            Ok(p) => p.to_app_err()?,
            Err(_) => return Err(create_error!(ProcessingTimeout)),
        };

        Ok(VideoProcessor {
            video: input,
            meta: metadata,
        })
    }

    async fn extract_frame(&self, time_secs: u64) -> Result<Image> {
        let frame = match timeout(
            EXTRACT_FRAME_TIMEOUT * (self.meta.video.duration_secs.round() as u32).max(1),
            media::video::extract_frame(Duration::from_secs(time_secs), &self.video),
        )
        .await
        {
            Ok(f) => {
                let f = f.to_app_err()?;
                Image::from_dynamic(f)
            }
            Err(_) => {
                return Err(create_error!(ProcessingTimeout));
            }
        };

        Ok(frame)
    }

    pub async fn generate_thumbnail(&self) -> Result<GeneratedImageVariant> {
        let frame = self.extract_frame(0).await?;
        ImageProcessor { image: frame }.generate_thumbnail()
    }

    pub async fn generate_loop_preview<'a>(
        &self,
        write_to: ReservedFile<'a>,
    ) -> Result<GeneratedVideoVariant<'a>> {
        let duration_ms = (self.meta.video.duration_secs * 1000.0).round() as u64;

        let fragment_duration_ms = LOOP_PREVIEW_FRAGMENT_DURATION_MS.min(duration_ms);
        let fragment_duration = Duration::from_millis(fragment_duration_ms);

        match timeout(
            TRANSCODINIG_TIMEOUT * (duration_ms / 1000).max(1) as u32,
            video::extract_video_fragment(
                &self.video,
                write_to.path(),
                ExtractVideoFragmentParams {
                    fragment: FragmentParams {
                        start: Duration::from_millis(0),
                        duration: fragment_duration,
                    },
                    frame_rate: None,
                    audio: AudioMode::Disabled,
                    output_resolution: ResizeParams::ForceWidth {
                        w: LOOP_PREVIEW_WIDTH,
                    },
                },
            ),
        )
        .await
        {
            Ok(f) => {
                f.to_app_err()?;
            }
            Err(_) => {
                return Err(create_error!(ProcessingTimeout));
            }
        }

        Ok(GeneratedVideoVariant {
            variant: MediaVariant::LoopPreview,
            mimetype: MimeType::Mp4,
            duration_milis: fragment_duration.as_millis() as u64,
            reserve: write_to,
        })
    }

    pub async fn extract_features(&self) -> Result<ExtractedFeatures> {
        let mid = self.meta.video.duration_secs.round() as u64 / 2;
        let frame = self.extract_frame(mid).await?;
        Ok(ImageProcessor { image: frame }.extract_features())
    }

    /// Returns a reference to the metadata of this [`VideoPipeline`]
    pub fn metadata(&self) -> &VideoMetadata {
        &self.meta
    }
}

// TODO: Replace processing via copying the original to a temp file
// with processing via streaming to an ffmpeg stdin pipe
pub async fn process_asset_as_video(ctx: &JobContext, asset: &AssetView) -> Result<()> {
    // Retrieves information about the original media file
    let original = get_original(ctx, asset.media_id()).await?;

    // Copies the video to a temporary directory for processing
    let path = StoragePath::from_str(&original.storage_path).to_app_err()?;
    let original_video = ctx.storage.use_local(&path).await?;

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
