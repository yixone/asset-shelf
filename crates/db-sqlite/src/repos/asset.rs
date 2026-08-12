use std::sync::Arc;

use db_core::{
    queries::asset::AssetQuery,
    repos::asset::AssetRepository,
    types::{
        DeleteResult, InsertResult, Pagination, UpdateResult,
        patch::{AssetFeaturesPatch, AssetPatch},
    },
};
use models::{
    entities::{Asset, AssetFeatures, AssetState},
    types::{AssetId, AssetsOrdering, Color},
};
use result::{Result, create_error, error::ResultExt};
use sqlx::QueryBuilder;

use crate::{
    driver::SqliteDatabase,
    helpers::{hydrate, queries},
};

pub struct SqliteAssetRepository {
    pub(crate) db: Arc<SqliteDatabase>,
}

#[async_trait::async_trait]
impl AssetRepository for SqliteAssetRepository {
    async fn insert(&self, asset: &Asset, features: &AssetFeatures) -> Result<InsertResult> {
        sqlx::query(
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
        .bind(asset.id)
        .bind(asset.state)
        .bind(&asset.media_id)
        .bind(asset.media_type)
        .bind(asset.created_at)
        .bind(asset.deleted_at)
        .bind(&asset.title)
        .bind(&asset.caption)
        .bind(&asset.source_url)
        .execute(self.db.exec())
        .await
        .to_app_err()?;

        let res = sqlx::query(
            "
            INSERT INTO asset_features (
                asset_id, p_hash, a_hash,
                width, height, accent_color
            )
            VALUES (
                ?, ?, ?,
                ?, ?, ?
            )
            ",
        )
        .bind(features.asset_id)
        .bind(features.p_hash)
        .bind(features.a_hash)
        .bind(features.width)
        .bind(features.height)
        .bind(features.accent_color)
        .execute(self.db.exec())
        .await
        .to_app_err()?;

        Ok(res.into())
    }

    async fn update(&self, id: AssetId, patch: AssetPatch) -> Result<UpdateResult<AssetQuery>> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE assets
            SET
            ",
        );

        patch.apply_sql(&mut qb);

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
    ) -> Result<UpdateResult<AssetQuery>> {
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

    async fn get_by_ids(&self, ids: &[AssetId]) -> Result<Vec<AssetQuery>> {
        let mut conn = self.db.acquire().await?;

        let assets = queries::get_assets(ids, &mut *conn).await?;

        hydrate::hydrate_assets(assets, &mut conn).await
    }

    async fn get_deleted(
        &self,
        pagination: Pagination,
        order: AssetsOrdering,
    ) -> Result<Vec<AssetQuery>> {
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
            AssetsOrdering::Newest => qb.push("ORDER BY a.created_at DESC"),
            AssetsOrdering::Oldest => qb.push("ORDER BY a.created_at ASC"),
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

    async fn get_random(&self) -> Result<AssetQuery> {
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

    async fn get_for_processing(&self, limit: u32) -> Result<Vec<AssetQuery>> {
        let mut conn = self.db.acquire().await?;

        let assets = sqlx::query_as(
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
        const COLOR_SHIFT: u8 = 75;

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
        .bind(aspect_ratio - 0.65)
        .bind(aspect_ratio + 0.65)
        .bind(p.limit())
        .bind(p.offset())
        .fetch_all(self.db.exec())
        .await
        .to_app_err()?;

        Ok(candidates)
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

    async fn list(&self, pagination: Pagination, order: AssetsOrdering) -> Result<Vec<AssetQuery>> {
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
