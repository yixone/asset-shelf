use std::sync::Arc;

use chrono::Utc;
use db_core::{
    ops::create_asset::CreateAssetOp,
    repos::asset::AssetRepository,
    types::{
        DeleteResult, Pagination, UpdateResult,
        patch::{AssetFeaturesPatch, AssetPatch},
    },
};
use models::{
    assets::{
        Asset, AssetFeatures,
        similar::SimilarAsset,
        view::{AssetView, SimilarAssetView},
    },
    types::{AssetId, AssetsOrdering, Color},
};
use result::{Result, create_error, error::ResultExt};
use sqlx::QueryBuilder;

use crate::{
    driver::SqliteDatabase, helpers::hydrate, ops::create_asset::SqliteCreateAssetOp, queries,
};

pub struct SqliteAssetRepository {
    pub db: Arc<SqliteDatabase>,
}

#[async_trait::async_trait]
impl AssetRepository for SqliteAssetRepository {
    async fn create_op<'a>(&'a self) -> Result<Box<dyn CreateAssetOp + 'a>> {
        let tx = self.db.begin().await?;
        let op = SqliteCreateAssetOp { tx };
        Ok(Box::new(op))
    }

    async fn update(&self, id: AssetId, patch: AssetPatch) -> Result<UpdateResult<AssetView>> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE assets
            SET
            ",
        );

        patch.apply_sql(&mut qb);

        qb.push(", updated_at = ");
        qb.push_bind(Utc::now());

        qb.push(" WHERE id = ");
        qb.push_bind(id);

        let res = qb.build().execute(self.db.exec()).await.to_app_err()?;
        if res.rows_affected() == 0 {
            return Ok(UpdateResult::NotFound);
        }

        let asset = self.get_by_id(id).await?;
        Ok(UpdateResult::Updated(asset))
    }

    async fn update_features(
        &self,
        id: AssetId,
        patch: AssetFeaturesPatch,
    ) -> Result<UpdateResult<AssetView>> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE asset_features
            SET
            ",
        );

        patch.apply_sql(&mut qb);

        qb.push(" WHERE asset_id = ");
        qb.push_bind(id);

        let res = qb.build().execute(self.db.exec()).await.to_app_err()?;
        if res.rows_affected() == 0 {
            return Ok(UpdateResult::NotFound);
        }

        let asset = self.get_by_id(id).await?;
        Ok(UpdateResult::Updated(asset))
    }

    async fn delete(&self, id: AssetId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM assets
            WHERE id = ?
            ",
        )
        .bind(id)
        .execute(self.db.exec())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn get_by_ids(&self, ids: &[AssetId]) -> Result<Vec<AssetView>> {
        let mut conn = self.db.acquire().await?;

        let assets = queries::asset::get_assets(ids, &mut *conn).await?;

        hydrate::hydrate_assets(assets, &mut conn).await
    }

    async fn get_deleted(
        &self,
        pagination: Pagination,
        order: AssetsOrdering,
    ) -> Result<Vec<AssetView>> {
        if pagination.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT a.* FROM assets AS a
            WHERE a.deleted_at IS NOT null
            ",
        );

        match order {
            AssetsOrdering::Newest => qb.push("ORDER BY a.deleted_at DESC"),
            AssetsOrdering::Oldest => qb.push("ORDER BY a.deleted_at ASC"),
        };
        pagination.apply_sql(&mut qb);

        let mut conn = self.db.acquire().await?;

        let assets = qb
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_app_err()?;

        hydrate::hydrate_assets(assets, &mut conn).await
    }

    async fn get_random(&self) -> Result<AssetView> {
        let mut conn = self.db.acquire().await?;

        let asset = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE deleted_at IS null
            ORDER BY RANDOM()
            ",
        )
        .fetch_optional(&mut *conn)
        .await
        .to_app_err()?
        .ok_or(create_error!(NotFound))?;

        hydrate::hydrate_assets(vec![asset], &mut conn)
            .await?
            .into_iter()
            .next()
            .ok_or(create_error!(NotFound))
    }

    async fn get_for_processing(&self, limit: u32) -> Result<Vec<AssetView>> {
        let mut conn = self.db.acquire().await?;

        let assets = sqlx::query_as(
            "
            SELECT a.*
            FROM assets AS a
            INNER JOIN asset_features
                AS af
                ON af.asset_id = a.id
            WHERE 
                a.deleted_at IS null 
                AND (
                    -- The asset has not yet been processed
                    a.state = 'Pending'

                    -- The asset was not processed due to an error or hang
                    OR (
                        a.state IN ('Processing', 'Failed')
                        AND (unixepoch('now') - unixepoch(a.updated_at)) >= ?
                    ) 

                    -- The asset was processed previously but currently lacks all the necessary fields
                    OR (
                        a.state = 'Ready' 
                        AND (
                            af.accent_color IS null OR
                            af.a_hash       IS null OR
                            af.p_hash       IS null OR
                            af.height       IS null OR
                            af.width        IS null
                        )
                    )
                )
            ORDER BY a.created_at ASC
            LIMIT ?
            ",
        )
        .bind(Asset::TIME_BEFORE_REPROCESSING.num_seconds())
        .bind(limit)
        .fetch_all(&mut *conn)
        .await
        .to_app_err()?;

        hydrate::hydrate_assets(assets, &mut conn).await
    }

    async fn get_for_similar_search(
        &self,
        color: Color,
        aspect_ratio: f32,
        p: Pagination,
    ) -> Result<Vec<AssetFeatures>> {
        let (red, green, blue) = color.rgb();
        const COLOR_SHIFT: u8 = 40;

        let candidates = sqlx::query_as(
            "
            SELECT af.*
            FROM asset_features AS af
            INNER JOIN assets AS a ON a.id = af.asset_id
            WHERE (
                -- RED
                ((af.accent_color >> 16) BETWEEN ? AND ?) OR
                -- GREEN
                ((af.accent_color >> 8 & 0xFF) BETWEEN ? AND ?) OR
                -- BLUE
                ((af.accent_color & 0xFF) BETWEEN ? AND ?) OR
                -- ASPECT RATIO
                ((af.width / af.height) BETWEEN ? AND ?)
            )
            ORDER BY a.created_at DESC
            LIMIT ?
            OFFSET ?
            ",
        )
        .bind(red.saturating_sub(COLOR_SHIFT))
        .bind(red.saturating_add(COLOR_SHIFT))
        .bind(green.saturating_sub(COLOR_SHIFT))
        .bind(green.saturating_add(COLOR_SHIFT))
        .bind(blue.saturating_sub(COLOR_SHIFT))
        .bind(blue.saturating_add(COLOR_SHIFT))
        .bind(aspect_ratio - 0.5)
        .bind(aspect_ratio + 0.5)
        .bind(p.limit())
        .bind(p.offset())
        .fetch_all(self.db.exec())
        .await
        .to_app_err()?;

        Ok(candidates)
    }

    async fn get_from_similar(&self, similar: Vec<SimilarAsset>) -> Result<Vec<SimilarAssetView>> {
        let mut pool = self.db.acquire().await?;
        hydrate::hydrate_similar(similar, &mut pool).await
    }

    async fn count_total(&mut self) -> Result<u64> {
        sqlx::query_scalar(
            "
            SELECT COUNT(id)
            FROM assets
            WHERE deleted_at IS null
            ",
        )
        .fetch_one(self.db.exec())
        .await
        .to_app_err()
    }

    async fn list(&self, pagination: Pagination, order: AssetsOrdering) -> Result<Vec<AssetView>> {
        if pagination.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT a.* FROM assets AS a
            WHERE a.deleted_at IS null
            ",
        );
        let mut query = qb.separated(" ");

        match order {
            AssetsOrdering::Newest => query.push("ORDER BY a.created_at DESC"),
            AssetsOrdering::Oldest => query.push("ORDER BY a.created_at ASC"),
        };
        pagination.apply_sql(&mut qb);

        let mut conn = self.db.acquire().await?;

        let assets = qb
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_app_err()?;

        hydrate::hydrate_assets(assets, &mut conn).await
    }
}
