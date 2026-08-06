use models::{
    entities::{MediaFile, MediaVariant},
    types::{MediaFileId, MediaId},
};
use result::{Result, error::ResultExt};
use sqlx::QueryBuilder;

use crate::{
    ops::{MediaFilesMaintenanceOps, MediaFilesReadOps, MediaFilesWriteOps},
    sqlite::SqliteExecutor,
    types::{DeleteResult, InsertResult, UpdateResult, patches::MediaFilePatch},
};

impl<T> MediaFilesReadOps for T
where
    T: SqliteExecutor,
{
    async fn get_media_files_by_ids(&mut self, ids: &[MediaFileId]) -> Result<Vec<MediaFile>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT * FROM media_files
            WHERE id IN
            ",
        );
        qb.push_tuples(ids, |mut qb, id| {
            qb.push_bind(id);
        });

        let media_files = qb
            .build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()?;
        Ok(media_files)
    }

    async fn get_media_files_by_groups(&mut self, media_ids: &[MediaId]) -> Result<Vec<MediaFile>> {
        if media_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT * FROM media_files
            WHERE media_id IN
            ",
        );
        qb.push_tuples(media_ids, |mut qb, id| {
            qb.push_bind(id);
        });

        let media_files = qb
            .build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()?;
        Ok(media_files)
    }

    async fn get_media_variant(
        &mut self,
        media_id: &MediaId,
        variant: MediaVariant,
    ) -> Result<Option<MediaFile>> {
        let media_file = sqlx::query_as(
            "
            SELECT * FROM media_files
            WHERE media_id = ?
                AND variant = ?
            ",
        )
        .bind(media_id)
        .bind(variant)
        .fetch_optional(self.executor())
        .await
        .to_app_err()?;
        Ok(media_file)
    }
}

impl<T> MediaFilesWriteOps for T
where
    T: SqliteExecutor,
{
    async fn insert_many_media_files(&mut self, mf: &[MediaFile]) -> Result<InsertResult> {
        if mf.is_empty() {
            return Ok(InsertResult::NoChanges);
        }

        let mut qb = QueryBuilder::new(
            "
            INSERT INTO media_files (
                id, storage_path, media_id,
                variant, created_at, size_bytes, mimetype,
                duration_milis
            )
            ",
        );

        qb.push_values(mf, |mut qb, mf| {
            qb.push_bind(&mf.id);
            qb.push_bind(&mf.storage_path);
            qb.push_bind(&mf.media_id);
            qb.push_bind(mf.variant);
            qb.push_bind(mf.created_at);
            qb.push_bind(mf.size_bytes);
            qb.push_bind(mf.mimetype);
            qb.push_bind(mf.duration_milis);
        });

        let res = qb.build().execute(self.executor()).await.to_app_err()?;
        Ok(res.into())
    }

    async fn update_media_file(
        &mut self,
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
            .fetch_optional(self.executor())
            .await
            .to_app_err()?;
        Ok(res.into())
    }

    async fn delete_many_media_files(&mut self, ids: &[MediaFileId]) -> Result<DeleteResult> {
        if ids.is_empty() {
            return Ok(DeleteResult::NoChanges);
        }

        let mut qb = QueryBuilder::new(
            "
            DELETE FROM media_files
            WHERE id IN 
            ",
        );
        qb.push_tuples(ids, |mut qb, id| {
            qb.push_bind(id);
        });

        let res = qb.build().execute(self.executor()).await.to_app_err()?;
        Ok(res.into())
    }
}

impl<T> MediaFilesMaintenanceOps for T
where
    T: SqliteExecutor,
{
    async fn count_media_files(&mut self, media_id: &MediaId) -> Result<i64> {
        let count = sqlx::query_scalar(
            "
            SELECT COUNT(id)
            FROM media_files
            WHERE media_id = ?
            ",
        )
        .bind(media_id)
        .fetch_one(self.executor())
        .await
        .to_app_err()?;
        Ok(count)
    }
}
