use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use chrono::Utc;
use db::{
    core::{
        patches::{AssetFeaturesPatch, AssetPatch},
        provider::{DatabaseConnector, TransactionUnit},
    },
    ops::{AssetFeaturesOps, AssetOps, MediaFilesOps},
};
use media::image::{Image, ImageFormat};
use models::{
    entities::{Asset, AssetState, MediaFile, MediaVariant},
    types::{AssetId, Color},
};
use result::{Result, create_error, error::ResultExt};
use storage::StoragePath;
use tokio_util::sync::CancellationToken;

use crate::{
    di::WorkerContext,
    queue::{EventsQueue, TasksSender},
    traits::{AbstractWorker, WorkerConfig},
};

/// Task for the media processing background worker
#[derive(Clone)]
pub enum MediaWorkerTask {
    /// Task to process asset media by [`AssetId`]:
    /// - Calculates media key parameters
    /// - Generates media variants
    ProcessAsset(AssetId),
}

/// Background media worker
pub struct MediaWorker {
    events: EventsQueue<MediaWorkerTask>,
    service: MediaWorkerService,
}

impl MediaWorker {
    /// Creates a new [`MediaWorker`] and returns it with [`TasksSender`] for worker's tasks
    pub fn new(ctx: WorkerContext) -> (TasksSender<MediaWorkerTask>, Self) {
        let queue = EventsQueue::new(1024);
        (
            queue.tx.clone(),
            Self {
                events: queue,
                service: MediaWorkerService { ctx },
            },
        )
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

        loop {
            tokio::select! {
                Some(task) = self.events.recv() => {
                    match task {
                        MediaWorkerTask::ProcessAsset(id) => self.service.process_asset_by_id(&id).await?,
                    }
                }
                _ = cancel.cancelled() => {
                    self.events.close();
                    break;
                }
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
        };

        // Processes the result of asset processing
        if let Err(e) = res {
            // In the event of a processing error,
            // it sets the state to 'failed' and propagates the error
            let mut conn = self.ctx.db.acquire().await?;
            let patch = AssetPatch::new().state(AssetState::Failed);
            conn.update_asset(&asset.id, patch).await?;

            return Err(e);
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
        let file = self.ctx.storage.get(&path).await?;

        // Decodes the image from the original file
        let img = Image::from_reader(file).await?;

        // Generates and saves a thumbnail
        let thumbnail = img.thumbnail(400).to_reader(ImageFormat::WebP)?;

        let thumbnail_file = self.ctx.storage.upload(thumbnail).await?;
        let thumbnail_path = thumbnail_file.commit_path(MediaVariant::Thumbnail.as_str());

        let thumbnail_media_file = MediaFile {
            id: self.ctx.flake.generate_id_as(),
            media_id: asset.media_id.clone(),
            variant: MediaVariant::Thumbnail,
            storage_path: thumbnail_path.to_string(),
            created_at: Utc::now(),
            size_bytes: thumbnail_file.file.size_bytes as i64,
            mimetype: thumbnail_file.file.mimetype,
        };

        // Retrieves the basic image parameters and features
        let (width, height) = img.dimension();

        let featured = img.prepare_features();
        let color = Color::from(featured.avg_color());
        let p_hash = featured.p_hash();
        let a_hash = featured.a_hash();
        drop(featured);

        let patch = AssetFeaturesPatch::new()
            .a_hash(Some(a_hash))
            .p_hash(Some(p_hash))
            .height(Some(height))
            .width(Some(width))
            .accent_color(Some(color));

        // Writes features to the database
        let mut tx = self.ctx.db.begin().await?;

        tx.insert_media_file(&thumbnail_media_file).await?;
        tx.update_asset_features(&asset.id, patch).await?;

        self.ctx
            .storage
            .commit(thumbnail_file, thumbnail_path)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
