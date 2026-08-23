use std::sync::Arc;

use db_core::{
    repos::media::MediaRepository,
    types::{DeleteResult, InsertResult, UpdateResult, patch::MediaFilePatch},
};
use models::{
    media::{Media, MediaFile, MediaVariant, view::MediaView},
    types::{MediaFileId, MediaId},
};
use result::{Result, create_error, error::ResultExt};
use sqlx::QueryBuilder;

use crate::{driver::SqliteDatabase, helpers::hydrate, queries};

pub struct SqliteMediaRepository {
    pub db: Arc<SqliteDatabase>,
}

#[async_trait::async_trait]
impl MediaRepository for SqliteMediaRepository {
    async fn insert(&self, media: &Media) -> Result<InsertResult> {
        queries::media::insert_media(media, self.db.exec()).await
    }

    async fn insert_file(&self, file: &MediaFile) -> Result<InsertResult> {
        queries::media::insert_media_file(file, self.db.exec()).await
    }

    async fn update_file(
        &self,
        id: &MediaFileId,
        patch: MediaFilePatch,
    ) -> Result<UpdateResult<MediaFile>> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE media_files
            SET
            ",
        );

        patch.apply_sql(&mut qb);

        qb.push(" WHERE id = ");
        qb.push_bind(id);

        qb.push(" RETURNING * ");

        let res = qb
            .build_query_as()
            .fetch_optional(self.db.exec())
            .await
            .to_app_err()?;

        Ok(res.into())
    }

    async fn delete_file(&self, id: &MediaFileId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM media_files
            WHERE id = ?
            ",
        )
        .bind(id)
        .execute(self.db.exec())
        .await
        .to_app_err()?;

        Ok(res.into())
    }

    async fn get_variant(&self, id: &MediaId, variant: MediaVariant) -> Result<MediaFile> {
        let media_file = sqlx::query_as(
            "
            SELECT * FROM media_files
            WHERE media_id = ?
                AND variant = ?
            ",
        )
        .bind(id)
        .bind(variant)
        .fetch_optional(self.db.exec())
        .await
        .to_app_err()?
        .ok_or(create_error!(NotFound))?;

        Ok(media_file)
    }

    async fn delete(&self, id: &MediaId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM media
            WHERE id = ?
            ",
        )
        .bind(id)
        .execute(self.db.exec())
        .await
        .to_app_err()?;

        Ok(res.into())
    }

    async fn get_by_ids(&self, ids: &[MediaId]) -> Result<Vec<MediaView>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT * FROM media
            WHERE id IN
            ",
        );
        qb.push_tuples(ids, |mut qb, id| {
            qb.push_bind(id);
        });

        let mut conn = self.db.acquire().await?;
        let media = qb
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_app_err()?;

        hydrate::hydrate_media(media, &mut conn).await
    }

    async fn get_orphans(&self, limit: u32) -> Result<Vec<MediaView>> {
        let mut conn = self.db.acquire().await?;

        let orphans = sqlx::query_as(
            "
            SELECT m.* 
            FROM media AS m
            LEFT JOIN assets AS a ON a.media_id = m.id
            GROUP BY m.id
            HAVING COUNT(a.id) = 0
            LIMIT ?
            ",
        )
        .bind(limit)
        .fetch_all(&mut *conn)
        .await
        .to_app_err()?;

        hydrate::hydrate_media(orphans, &mut conn).await
    }
}
