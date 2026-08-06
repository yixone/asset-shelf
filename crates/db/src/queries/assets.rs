use models::{
    entities::{Asset, AssetState},
    types::{AssetId, AssetsOrdering},
};
use result::{Result, error::ResultExt};
use sqlx::QueryBuilder;

use crate::{
    ops::{AssetsMaintenanceOps, AssetsReadOps, AssetsWriteOps},
    sqlite::SqliteExecutor,
    types::{DeleteResult, InsertResult, Pagination, UpdateResult, patches::AssetPatch},
};

impl<T> AssetsReadOps for T
where
    T: SqliteExecutor,
{
    async fn get_assets_by_ids(&mut self, ids: &[AssetId]) -> Result<Vec<Asset>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT * FROM assets
            WHERE id IN
            ",
        );
        qb.push_tuples(ids, |mut qb, id| {
            qb.push_bind(id);
        });

        let assets = qb
            .build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()?;
        Ok(assets)
    }

    async fn get_deleted_assets(&mut self, p: Pagination, o: AssetsOrdering) -> Result<Vec<Asset>> {
        if p.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT a.* FROM assets AS a
            WHERE a.deleted_at IS NOT null
            ",
        );

        match o {
            AssetsOrdering::Newest => qb.push("ORDER BY a.created_at DESC"),
            AssetsOrdering::Oldest => qb.push("ORDER BY a.created_at ASC"),
        };
        p.apply_sql(&mut qb);

        let assets = qb
            .build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()?;
        Ok(assets)
    }

    async fn list_assets(&mut self, p: Pagination, o: AssetsOrdering) -> Result<Vec<Asset>> {
        if p.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT a.* FROM assets AS a
            WHERE a.deleted_at IS null
            ",
        );
        let mut query = qb.separated(" ");

        match o {
            AssetsOrdering::Newest => query.push("ORDER BY a.created_at DESC"),
            AssetsOrdering::Oldest => query.push("ORDER BY a.created_at ASC"),
        };
        p.apply_sql(&mut qb);

        let assets = qb
            .build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()?;
        Ok(assets)
    }

    async fn random_assets(&mut self, limit: u32) -> Result<Vec<Asset>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let assets = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE deleted_at IS null
            ORDER BY RANDOM()
            LIMIT ?
            ",
        )
        .bind(limit)
        .fetch_all(self.executor())
        .await
        .to_app_err()?;
        Ok(assets)
    }

    async fn count_assets(&mut self) -> Result<u64> {
        let count = sqlx::query_scalar(
            "
            SELECT COUNT(id)
            FROM assets
            WHERE deleted_at IS null
            ",
        )
        .fetch_one(self.executor())
        .await
        .to_app_err()?;

        Ok(count)
    }
}

impl<T> AssetsWriteOps for T
where
    T: SqliteExecutor,
{
    async fn insert_asset(&mut self, a: &Asset) -> Result<InsertResult> {
        let res = sqlx::query(
            "
            INSERT INTO assets (
                id, state,
                media_id, media_type,
                created_at, deleted_at,
                title, caption, source_url
            )
            VALUES (
                ?, ?,
                ?, ?,
                ?, ?,
                ?, ?, ?
            )
            ",
        )
        .bind(a.id)
        .bind(a.state)
        .bind(&a.media_id)
        .bind(a.media_type)
        .bind(a.created_at)
        .bind(a.deleted_at)
        .bind(&a.title)
        .bind(&a.caption)
        .bind(&a.source_url)
        .execute(self.executor())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn update_asset(
        &mut self,
        id: &AssetId,
        patch: AssetPatch,
    ) -> Result<UpdateResult<Asset>> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE assets
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

    async fn delete_asset(&mut self, id: &AssetId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM assets
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

impl<T> AssetsMaintenanceOps for T
where
    T: SqliteExecutor,
{
    async fn get_unprocessed_assets(&mut self, limit: u32) -> Result<Vec<Asset>> {
        let unprocessed = sqlx::query_as(
            "
            SELECT a.*
            FROM assets AS a
            INNER JOIN asset_features
                AS af
                ON af.asset_id = a.id
            WHERE
                a.state != ? AND 
            (
                af.accent_color IS null OR
                af.a_hash       IS null OR
                af.p_hash       IS null OR
                af.height       IS null OR
                af.width        IS null
            )
            ORDER BY a.created_at ASC
            LIMIT ?
            ",
        )
        .bind(AssetState::Processing)
        .bind(limit)
        .fetch_all(self.executor())
        .await
        .to_app_err()?;

        Ok(unprocessed)
    }
}
