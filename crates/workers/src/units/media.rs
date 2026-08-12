use std::{
    collections::HashSet,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::Utc;
use db::{
    queries::asset::AssetQuery,
    types::patch::{AssetFeaturesPatch, MediaFilePatch},
};
use events::{AssetCreatedEvent, EventBus};
use media::{
    image::{Image, ImageDecoder, ImageFormat},
    video::{
        self, ExtractVideoFragmentParams, FragmentParams, input::MediaInput, types::AudioMode,
    },
};
use mimetype::{MimeKind, MimeType};
use models::{
    entities::{AssetState, MediaFile, MediaVariant},
    types::{AssetId, Color, MediaId},
};
use result::{ErrorKind, Result, error::ResultExt};
use storage::{StoragePath, global::GlobalPathData};
use tokio::io::AsyncRead;
use tokio_util::sync::CancellationToken;

use crate::{
    WorkerContext,
    worker::{AbstractWorker, WorkerConfig},
};

/// Background media worker
pub struct MediaWorker {
    service: MediaWorkerService,
    events: Arc<EventBus>,
}

impl MediaWorker {
    /// Creates a new [`MediaWorker`] and returns it with [`TasksSender`] for worker's tasks
    pub fn new(ctx: WorkerContext) -> Self {
        Self {
            events: ctx.events.clone(),
            service: MediaWorkerService { ctx },
        }
    }
}

#[async_trait::async_trait]
impl AbstractWorker for MediaWorker {
    /// Returns the background service configuration
    fn cfg(&self) -> WorkerConfig {
        WorkerConfig {
            name: "MediaWorker",
            restart_delay: Duration::from_secs(30),
            allow_restart: true,
        }
    }

    /// Executes the service runtime
    async fn runtime(&mut self, cancel: CancellationToken) -> Result<()> {
        self.service.process_unprocessed_media().await?;

        let mut on_asset_created = self.events.subscribe::<AssetCreatedEvent>();

        loop {
            tokio::select! {
                Ok(new_asset) = on_asset_created.recv() => {
                    self.service.process_asset_by_id(&new_asset.asset).await?
                },
                _ = cancel.cancelled() => break
            }
        }

        Ok(())
    }
}

/// Service for a [`MediaWorker`]
struct MediaWorkerService {
    ctx: WorkerContext,
}

impl MediaWorkerService {
    /// Processes media of pending assets or assets lacking certain features
    async fn process_unprocessed_media(&self) -> Result<usize> {
        let mut processed = 0;

        // Retrieves unprocessed assets as long as there are any in the database
        loop {
            // Receives unprocessed assets
            let unprocessed = self.ctx.db.assets.get_for_processing(50).await?;

            // Triggers processing for all unprocessed assets
            for a in &unprocessed {
                self.process_asset_media(a).await?;
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
    async fn process_asset_by_id(&self, id: &AssetId) -> Result<()> {
        // Retrieves an Asset by ID
        let asset = {
            let asset = self.ctx.db.assets.get_by_id(*id).await;
            match asset {
                Ok(a) => a,
                Err(e) if matches!(e.kind(), ErrorKind::NotFound) => return Ok(()),
                Err(e) => return Err(e),
            }
        };

        // Calls processing for the asset
        self.process_asset_media(&asset).await
    }

    /// Executes the common media processing pipeline for the specified [`Asset`]
    async fn process_asset_media(&self, asset: &AssetQuery) -> Result<()> {
        // Check that the received asset is not marked as being processed
        if asset.inner.state == AssetState::Processing {
            return Ok(());
        }

        // Setting the asset status to `processing`
        {
            if self
                .ctx
                .db
                .assets
                .update_state(asset.inner.id, AssetState::Processing)
                .await?
                .no_changes()
            {
                return Ok(());
            }
        }

        // Records the time when processing began
        let t0 = Instant::now();

        // Executes a processing pipeline suitable for the asset type
        let res = match asset.inner.media_type {
            MimeKind::Image => self.process_asset_as_image(asset).await,
            MimeKind::Video => self.process_asset_as_video(asset).await,
        };

        // Processes the result of asset processing
        if let Err(e) = res {
            // In the event of a processing error,
            // it sets the state to 'failed' and propagates the error
            self.ctx
                .db
                .assets
                .update_state(asset.inner.id, AssetState::Failed)
                .await?;

            return Err(e);
        } else {
            self.ctx
                .db
                .assets
                .update_state(asset.inner.id, AssetState::Ready)
                .await?
                .no_changes()
        };

        tracing::info!(
            id = ?asset.inner.id, elapsed = t0.elapsed().as_millis(), m_type = ?asset.inner.media_type,
            "MediaWorker: asset media processed"
        );
        Ok(())
    }

    /// Executes the media image processing pipeline for the specified [`Asset`]
    async fn process_asset_as_image(&self, asset: &AssetQuery) -> Result<()> {
        tracing::info!("Processing {} as image", asset.inner.id);

        // Retrieves information about the original media file
        let original = self
            .ctx
            .db
            .media
            .get_variant(&asset.inner.media_id, MediaVariant::Original)
            .await?;

        // Retrieves the original file from the storage
        let path = StoragePath::from_str(&original.storage_path).to_app_err()?;
        let file = self.ctx.storage.open(&path).await?;

        // Decodes the image from the original file
        let img = ImageDecoder::from_async_read(file).await?;

        // Generates different variants for an image
        self.generate_image_variants(asset, &img).await?;

        // Retrieves the basic image parameters and features
        let (width, height) = img.dimension();

        let featured = img.features();
        let color = Color::from(featured.avg_color());
        let p_hash = featured.p_hash();
        let a_hash = featured.a_hash();

        let patch = AssetFeaturesPatch::new()
            .a_hash(Some(a_hash))
            .p_hash(Some(p_hash))
            .height(Some(height))
            .width(Some(width))
            .accent_color(Some(color));

        // Writes features to the database
        self.ctx
            .db
            .assets
            .update_features(asset.inner.id, patch)
            .await?;

        Ok(())
    }

    /// Executes the media video processing pipeline for the specified [`Asset`]
    async fn process_asset_as_video(&self, asset: &AssetQuery) -> Result<()> {
        tracing::info!("Processing {} as video", asset.inner.id);

        // Retrieves information about the original media file
        let original = self
            .ctx
            .db
            .media
            .get_variant(&asset.inner.media_id, MediaVariant::Original)
            .await?;

        // Copies the video to a temporary directory for processing
        let path = StoragePath::from_str(&original.storage_path).to_app_err()?;
        let original_video = self.ctx.storage.open_local(&path).await?;

        // Opens the video as a cassette input
        let video = video::input::MediaInput::try_new(original_video.path()).to_app_err()?;

        // Extracts metadata from video
        let metadata = media::video::probe_video(&video).await.to_app_err()?;
        let duration_milis = (metadata.video.duration_secs * 1000.0) as i64;
        let res = metadata.video.resolution;
        let (width, height) = (res.width, res.height);

        // Extract frame and generates thumbnail
        let frame = media::video::extract_frame(Duration::from_secs(0), &video)
            .await
            .to_app_err()?;
        let frame = Image::from_dynamic(frame);

        // Generates video variants
        self.generate_video_variants(asset, duration_milis, &frame, &video)
            .await?;

        // Extract features from thumbnail
        let featured = frame.features();
        let color = Color::from(featured.avg_color());
        let p_hash = featured.p_hash();
        let a_hash = featured.a_hash();

        // Writes features to the database
        let patch = AssetFeaturesPatch::new()
            .a_hash(Some(a_hash))
            .p_hash(Some(p_hash))
            .height(Some(height))
            .width(Some(width))
            .accent_color(Some(color));
        let f_patch = MediaFilePatch::new().duration_milis(Some(duration_milis));

        self.ctx
            .db
            .assets
            .update_features(asset.inner.id, patch)
            .await?;
        self.ctx.db.media.update_file(&original.id, f_patch).await?;

        Ok(())
    }

    /// Checks which variants already exist for the specified asset
    async fn get_exists_media_variants(&self, asset: &AssetQuery) -> Result<HashSet<MediaVariant>> {
        let stored_variants = &asset.media;
        let mut variants = HashSet::with_capacity(stored_variants.len());

        for v in stored_variants {
            variants.insert(v.variant);
        }

        Ok(variants)
    }

    async fn generate_video_variants(
        &self,
        asset: &AssetQuery,
        video_duration_milis: i64,
        frame: &Image,
        video: &MediaInput,
    ) -> Result<()> {
        // Checks which variants already exist for the specified asset
        let variants = self.get_exists_media_variants(asset).await?;

        if !variants.contains(&MediaVariant::Thumbnail) {
            let thumbnail = frame
                .thumbnail(400)
                .reader(ImageFormat::WebP { quality: 80 })?;
            self.store_variant(
                &asset.inner.media_id,
                MimeType::Webp,
                MediaVariant::Thumbnail,
                thumbnail,
            )
            .await?;
        }

        if !variants.contains(&MediaVariant::LoopPreview) {
            let reserve = self.ctx.storage.reserve(GlobalPathData::new(
                &asset.inner.media_id.to_string(),
                MediaVariant::LoopPreview.as_str(),
            ));

            let duration = Duration::from_millis((5000).min(video_duration_milis as u64));

            video::extract_video_fragment(
                video,
                reserve.path(),
                ExtractVideoFragmentParams {
                    fragment: FragmentParams {
                        start: Duration::from_millis(0),
                        duration,
                    },
                    frame_rate: None,
                    audio: AudioMode::Disabled,
                    output_resolution: Some((1280, 720)),
                },
            )
            .await
            .to_app_err()?;

            let file = reserve.publish().await?;

            let variant_media_file = MediaFile {
                id: self.ctx.flake.get_id_as(),
                media_id: asset.inner.media_id.clone(),
                variant: MediaVariant::LoopPreview,
                storage_path: file.path.to_string(),
                created_at: Utc::now(),
                size_bytes: file.size_bytes as i64,
                mimetype: MimeType::Mp4,
                duration_milis: Some(duration.as_millis() as i64),
            };

            self.ctx.db.media.insert_file(&variant_media_file).await?;
            tracing::info!(
                "MediaWorker: loop preview generated and saved for media {}",
                asset.inner.media_id
            );
        }

        Ok(())
    }

    async fn generate_image_variants(&self, asset: &AssetQuery, img: &Image) -> Result<()> {
        // Checks which variants already exist for the specified asset
        let variants = self.get_exists_media_variants(asset).await?;

        // Generates and saves a thumbnail
        if !variants.contains(&MediaVariant::Thumbnail) {
            let thumbnail = img
                .thumbnail(400)
                .reader(ImageFormat::WebP { quality: 80 })?;
            self.store_variant(
                &asset.inner.media_id,
                MimeType::Webp,
                MediaVariant::Thumbnail,
                thumbnail,
            )
            .await?;
        }

        Ok(())
    }

    async fn store_variant<R>(
        &self,
        media: &MediaId,
        mimetype: MimeType,
        variant: MediaVariant,
        variant_bytes: R,
    ) -> Result<()>
    where
        R: AsyncRead + Send + Unpin,
    {
        let variant_file = self
            .ctx
            .storage
            .upload(
                GlobalPathData::new(&media.to_string(), variant.as_str()),
                variant_bytes,
                |_| Ok(()),
            )
            .await?;

        let variant_media_file = MediaFile {
            id: self.ctx.flake.get_id_as(),
            media_id: media.clone(),
            variant: MediaVariant::Thumbnail,
            storage_path: variant_file.global_path().to_string(),
            created_at: Utc::now(),
            size_bytes: variant_file.size_bytes as i64,
            mimetype,
            duration_milis: None,
        };

        self.ctx.db.media.insert_file(&variant_media_file).await?;
        variant_file.commit().await?;

        tracing::info!("MediaWorker: {variant} generated and saved for media: {media}");
        Ok(())
    }
}
