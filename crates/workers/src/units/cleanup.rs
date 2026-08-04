use std::{str::FromStr, sync::Arc, time::Duration};

use chrono::Utc;
use db::{
    database::{DatabaseProvider, DatabaseTransaction},
    ops::{AssetOps, MediaFilesOps, MediaOps},
    types::Pagination,
};

use events::{AssetDeletedEvent, EventBus};
use join::JoinBuilder;
use models::{
    bulk::BulkIds,
    entities::{Asset, Media, MediaFile},
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
        let mut conn = self.ctx.db.acquire().await?;
        let Some(media) = conn.get_media(id).await? else {
            return Ok(());
        };
        let files = conn.get_media_files(&media.id).await?;

        self.delete_media(&media, files).await
    }

    async fn cleanup_orphaned(&self) -> Result<usize> {
        let mut deleted = 0;

        let mut conn = self.ctx.db.acquire().await?;
        loop {
            let media = conn.get_orphans_media(50).await?;
            let files = conn.get_media_files_bulk(&media.ids()).await?;

            let count = media.len();

            for (m, f) in JoinBuilder::new(media).with_group(files, |m| m).build() {
                self.delete_media(&m, f).await?;
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

        let mut conn = self.ctx.db.acquire().await?;
        loop {
            let marked = conn
                .get_deleted_assets(Pagination::new(50, 0), AssetsOrdering::Oldest)
                .await?;

            let mut processed = 0;

            for a in &marked {
                let deleted = a
                    .deleted_at
                    .expect("asset_repo.get_deleted_list(..) returned asset without deleted_at");

                if (now - deleted) >= retention_time {
                    self.delete_asset(a).await?;
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
        let mut conn = self.ctx.db.acquire().await?;
        conn.delete_asset(&asset.id).await?;
        Ok(())
    }

    async fn delete_media(&self, media: &Media, files: Vec<MediaFile>) -> Result<()> {
        for f in &files {
            self.delete_media_file(f).await?;
        }

        let mut tx = self.ctx.db.begin().await?;

        tx.delete_media_file_bulk(&files.ids()).await?;
        tx.delete_media(&media.id).await?;

        tx.commit().await?;

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
