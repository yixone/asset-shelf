use models::{entities::Media, types::MediaId};
use result::{Result, error::ResultExt};
use sqlx::QueryBuilder;

use crate::{
    ops::{MediaMaintenanceOps, MediaReadOps, MediaWriteOps},
    sqlite::SqliteExecutor,
    types::result::{DeleteResult, InsertResult},
};

impl<T> MediaReadOps for T
where
    T: SqliteExecutor,
{
    async fn get_media_by_ids(&mut self, ids: &[MediaId]) -> Result<Vec<Media>> {
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

        let media = qb
            .build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()?;
        Ok(media)
    }
}

impl<T> MediaWriteOps for T
where
    T: SqliteExecutor,
{
    async fn insert_media(&mut self, m: &Media) -> Result<InsertResult> {
        let res = sqlx::query(
            "
            INSERT INTO media (
                id, created_at
            )
            VALUES (
                ?, ?
            )
            ",
        )
        .bind(&m.id)
        .bind(m.created_at)
        .execute(self.executor())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn delete_media(&mut self, id: &MediaId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM media
            WHERE id = ?
            ",
        )
        .bind(id)
        .execute(self.executor())
        .await
        .to_app_err()?;
        Ok(res.into())
    }
}

impl<T> MediaMaintenanceOps for T
where
    T: SqliteExecutor,
{
    async fn get_orphans_media(&mut self, limit: u32) -> Result<Vec<Media>> {
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
        .fetch_all(self.executor())
        .await
        .to_app_err()?;

        Ok(orphans)
    }
}
