use std::{str::FromStr, sync::Arc, time::Duration};

use chrono::Utc;
use db::types::Pagination;

use events::{AssetDeletedEvent, EventBus};
use models::{
    assets::Asset,
    media::{MediaFile, view::MediaView},
    types::{AssetsOrdering, MediaId},
};
use result::{Result, error::ResultExt};
use storage::StoragePath;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::{
    WorkerContext,
    worker::{AbstractWorker, WorkerConfig},
};

pub struct CleanupWorker {
    service: CleanupWorkerService,
    events: Arc<EventBus>,
}

impl CleanupWorker {
    pub fn new(ctx: WorkerContext) -> Self {
        CleanupWorker {
            events: ctx.events.clone(),
            service: CleanupWorkerService { ctx },
        }
    }
}

#[async_trait::async_trait]
impl AbstractWorker for CleanupWorker {
    /// Returns the background service configuration
    fn cfg(&self) -> WorkerConfig {
        WorkerConfig {
            name: "CleanupWorker",
            restart_delay: Duration::from_secs(30),
            allow_restart: true,
        }
    }

    /// Executes the service runtime
    async fn runtime(&mut self, cancel: CancellationToken) -> Result<()> {
        let mut cleanup_interval = interval(Duration::from_mins(90));

        let mut on_media_removed = self.events.subscribe::<AssetDeletedEvent>();

        loop {
            tokio::select! {
                Ok(e) = on_media_removed.recv() => {
                    self.service.remove_media_by_id(&e.media).await?
                }
                _ = cleanup_interval.tick() => {
                    tracing::info!("CleanupWorker: Starting interval auto-cleaning");

                    let orphaned = self.service.cleanup_orphaned().await?;
                    tracing::info!("CleanupWorker: {orphaned} orphaned Media removed");

                    let del_assets = self.service.cleanup_deleted_assets().await?;
                    tracing::info!("CleanupWorker: {del_assets} assets marked as deleted have been deleted");
                }
                _ = cancel.cancelled() => {
                    break;
                }
            }
        }

        Ok(())
    }
}

/// Service for a [`CleanupWorker`]
struct CleanupWorkerService {
    ctx: WorkerContext,
}

impl CleanupWorkerService {
    async fn remove_media_by_id(&self, id: &MediaId) -> Result<()> {
        let media = self.ctx.db.media.get_by_id(id).await?;
        self.delete_media(media).await
    }

    async fn cleanup_orphaned(&self) -> Result<usize> {
        let mut deleted = 0;

        loop {
            let media = self.ctx.db.media.get_orphans(50).await?;

            let count = media.len();

            for m in media {
                self.delete_media(m).await?;
                deleted += 1;
            }

            if count < 50 {
                break;
            }
        }

        Ok(deleted)
    }

    async fn cleanup_deleted_assets(&self) -> Result<usize> {
        let mut deleted = 0;
        let now = Utc::now();
        let retention_time = chrono::Duration::days(30);

        loop {
            let marked = self
                .ctx
                .db
                .assets
                .get_deleted(Pagination::new(50, 0), AssetsOrdering::Oldest)
                .await?;

            let mut processed = 0;

            for a in &marked {
                let deleted = a
                    .inner
                    .deleted_at
                    .expect("asset_repo.get_deleted_list(..) returned asset without deleted_at");

                if (now - deleted) >= retention_time {
                    self.delete_asset(&a.inner).await?;
                    processed += 1;
                } else {
                    tracing::info!("Reached an asset that was deleted less than 30 days ago");
                    break;
                }
            }

            deleted += processed;
            if processed != marked.len() || marked.len() < 50 {
                break;
            }
        }
        Ok(deleted)
    }

    async fn delete_asset(&self, asset: &Asset) -> Result<()> {
        self.ctx.db.assets.delete(asset.id).await?;
        Ok(())
    }

    async fn delete_media(&self, media: MediaView) -> Result<()> {
        for f in &media.files {
            self.ctx.db.media.delete_file(&f.id).await?;
            self.delete_media_file(f).await?;
        }

        self.ctx.db.media.delete(&media.inner.id).await?;

        Ok(())
    }

    async fn delete_media_file(&self, file: &MediaFile) -> Result<()> {
        let path = StoragePath::from_str(&file.storage_path).to_app_err()?;
        if self.ctx.storage.remove_safely(&path).await {
            tracing::info!(path = ?file.storage_path, "CleanupWorker: Removed storage media file");
        }
        Ok(())
    }
}
