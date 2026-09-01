// !FIXME: Extract the processing code from the `jobs` module

use std::str::FromStr;
use std::time::Instant;
use std::{path::Path, time::Duration};

use chrono::Utc;
use db::types::UpdateResult;
use db::types::patch::{AssetFeaturesPatch, MediaFilePatch};
use mimetype::{MimeKind, MimeType};
use models::assets::AssetState;
use models::assets::view::AssetView;
use models::media::MediaFile;
use models::types::{Color, MediaId};
use models::{media::MediaVariant, types::AssetId};
use result::{Result, create_error, error::ResultExt};
use storage::StoragePath;
use storage::global::GlobalPathData;
use tokio::io::AsyncRead;

use image::ImageProcessor;
use video::VideoProcessor;

use crate::JobContext;

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

async fn process_asset_as_image(ctx: &JobContext, asset: &AssetView) -> Result<()> {
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
        store::store_image_variant(ctx, thumbnail, asset.media_id()).await?;
    }

    // Retrieves the basic image parameters and features
    let features = processor.extract_features();

    // Writes features to the database
    let patch = features.into();
    ctx.db.assets.update_features(asset.inner.id, patch).await?;

    Ok(())
}

// TODO: Replace processing via copying the original to a temp file
// with processing via streaming to an ffmpeg stdin pipe
async fn process_asset_as_video(ctx: &JobContext, asset: &AssetView) -> Result<()> {
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
        store::store_image_variant(ctx, thumbnail, asset.media_id()).await?;
    }

    // Generates loop preview
    if !variants.contains(&MediaVariant::LoopPreview) {
        let reserve = ctx.storage.reserve(GlobalPathData::new(
            &asset.inner.media_id.to_string(),
            MediaVariant::LoopPreview.as_str(),
        ));

        let loop_preview = processor.generate_loop_preview(reserve).await?;
        store::store_video_variant(ctx, loop_preview, asset.media_id()).await?;
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
    ctx: &JobContext,
    id: AssetId,
    state: AssetState,
) -> Result<UpdateResult<AssetView>> {
    ctx.db.assets.update_state(id, state).await
}

async fn get_original(ctx: &JobContext, id: &MediaId) -> Result<MediaFile> {
    ctx.db.media.get_variant(id, MediaVariant::Original).await
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

mod image {
    use media::image::{Image, ImageDecoder, ImageFormat, ImageReader};

    use super::*;

    const THUMBNAIL_WIDTH: u32 = 400;
    const THUMBNAIL_QUALITY: usize = 80;

    pub struct GeneratedImageVariant {
        pub variant: MediaVariant,
        pub mimetype: MimeType,
        pub reader: ImageReader,
    }

    pub struct ImageProcessor {
        image: Image,
    }

    impl ImageProcessor {
        pub fn new(img: Image) -> Self {
            ImageProcessor { image: img }
        }

        pub async fn decode<R>(data: R) -> Result<Self>
        where
            R: AsyncRead + Send + Unpin,
        {
            let img = ImageDecoder::from_async_read(data).await?;
            Ok(ImageProcessor::new(img))
        }

        /// Extracts features from an image and returns them as [`ExtractedFeatures`]
        pub fn extract_features(&self) -> ExtractedFeatures {
            let (width, height) = self.image.dimension();

            let featured = self.image.features();
            let color = Color::from(featured.avg_color());
            let p_hash = featured.p_hash();
            let a_hash = featured.a_hash();

            ExtractedFeatures {
                a_hash,
                p_hash,
                color,
                width,
                height,
            }
        }

        /// Generates a thumbnail for the current image and returns it as a reader
        pub fn generate_thumbnail(&self) -> Result<GeneratedImageVariant> {
            let thumbnail = self
                .image
                .thumbnail(THUMBNAIL_WIDTH)
                .reader(ImageFormat::WebP {
                    quality: THUMBNAIL_QUALITY,
                })?;

            Ok(GeneratedImageVariant {
                variant: MediaVariant::Thumbnail,
                mimetype: MimeType::Webp,
                reader: thumbnail,
            })
        }
    }
}

mod video {
    use media::{
        image::Image,
        video::{
            self, ExtractVideoFragmentParams, FragmentParams, ResizeParams,
            input::MediaInput,
            types::{AudioMode, VideoMetadata},
        },
    };
    use storage::files::ReservedFile;
    use tokio::time::timeout;

    use super::image::{GeneratedImageVariant, ImageProcessor};
    use super::*;

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
            ImageProcessor::new(frame).generate_thumbnail()
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
            Ok(ImageProcessor::new(frame).extract_features())
        }

        /// Returns a reference to the metadata of this [`VideoPipeline`]
        pub fn metadata(&self) -> &VideoMetadata {
            &self.meta
        }
    }
}

mod store {
    use models::{media::MediaFile, types::MediaId};
    use storage::global::GlobalPathData;

    use super::*;
    use super::{image::GeneratedImageVariant, video::GeneratedVideoVariant};

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
}
