use std::{
    collections::HashSet,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::Utc;
use db::{
    database::DatabaseProvider,
    ops::{AssetFeaturesOps, AssetOps, MediaFilesOps},
    types::patches::{AssetFeaturesPatch, AssetPatch},
};
use events::{AssetCreatedEvent, EventBus};
use media::image::{Image, ImageDecoder, ImageFormat};
use mimetype::MimeType;
use models::{
    entities::{Asset, AssetState, MediaFile, MediaVariant},
    types::{AssetId, Color, MediaId},
};
use result::{Result, create_error, error::ResultExt};
use storage::{StoragePath, global::GlobalPathData};
use tokio::io::AsyncRead;
use tokio_util::sync::CancellationToken;

use crate::{
    WorkerContext,
    worker::{AbstractWorker, WorkerConfig},
};

const MEDIA_NAMESPACE: &str = "media";

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
        let mut conn = self.ctx.db.acquire().await?;
        loop {
            // Receives unprocessed assets
            let unprocessed = conn.get_unprocessed_assets(50).await?;

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
            let mut conn = self.ctx.db.acquire().await?;
            let Some(asset) = conn.get_asset(id).await? else {
                // TODO: ADD TRACING!
                return Ok(());
            };
            asset
        };

        // Calls processing for the asset
        self.process_asset_media(&asset).await
    }

    /// Executes the common media processing pipeline for the specified [`Asset`]
    async fn process_asset_media(&self, asset: &Asset) -> Result<()> {
        // Check that the received asset is not marked as being processed
        if asset.state == AssetState::Processing {
            return Ok(());
        }

        // Setting the asset status to `processing`
        {
            let mut conn = self.ctx.db.acquire().await?;
            if conn
                .update_asset(&asset.id, AssetPatch::new().state(AssetState::Processing))
                .await?
                .no_changes()
            {
                return Ok(());
            }
        }

        // Records the time when processing began
        let t0 = Instant::now();

        // Executes a processing pipeline suitable for the asset type
        let res = match asset.media_type {
            mimetype::MimeKind::Image => self.process_asset_as_image(asset).await,
            mimetype::MimeKind::Video => {
                // TODO!
                Ok(())
            }
        };

        // Processes the result of asset processing
        if let Err(e) = res {
            // In the event of a processing error,
            // it sets the state to 'failed' and propagates the error
            let mut conn = self.ctx.db.acquire().await?;
            let patch = AssetPatch::new().state(AssetState::Failed);
            conn.update_asset(&asset.id, patch).await?;

            return Err(e);
        } else {
            let mut conn = self.ctx.db.acquire().await?;
            let patch = AssetPatch::new().state(AssetState::Ready);
            conn.update_asset(&asset.id, patch).await?;
        };

        tracing::info!(
            id = ?asset.id, elapsed = t0.elapsed().as_millis(), m_type = ?asset.media_type,
            "MediaWorker: asset media processed"
        );
        Ok(())
    }

    /// Executes the media image processing pipeline for the specified [`Asset`]
    async fn process_asset_as_image(&self, asset: &Asset) -> Result<()> {
        // Retrieves information about the original media file
        let original = {
            let mut conn = self.ctx.db.acquire().await?;
            conn.get_media_variant(&asset.media_id, MediaVariant::Original)
                .await?
                .ok_or(create_error!(NotFound))?
        };

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
        let mut conn = self.ctx.db.acquire().await?;
        conn.update_asset_features(&asset.id, patch).await?;

        Ok(())
    }

    async fn generate_image_variants(&self, asset: &Asset, img: &Image) -> Result<()> {
        // Checks which variants already exist for the specified asset
        let variants = {
            let mut conn = self.ctx.db.acquire().await?;
            let stored_variants = conn.get_media_files(&asset.media_id).await?;
            let mut variants = HashSet::with_capacity(stored_variants.len());

            for v in stored_variants {
                variants.insert(v.variant);
            }
            variants
        };

        // Generates and saves a thumbnail
        if !variants.contains(&MediaVariant::Thumbnail) {
            let thumbnail = img
                .thumbnail(400)
                .reader(ImageFormat::WebP { quality: 80 })?;
            self.store_variant(
                &asset.media_id,
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
        };

        let mut conn = self.ctx.db.acquire().await?;
        conn.insert_media_file(&variant_media_file).await?;

        variant_file.commit().await?;

        tracing::info!("MediaWorker: {variant} generated and saved for media:{media}");
        Ok(())
    }
}
