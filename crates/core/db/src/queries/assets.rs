use models::{
    entities::{Asset, AssetFeatures},
    types::{AssetId, AssetsOrdering},
};
use sqlx::QueryBuilder;

use crate::{
    core::{
        Result,
        pagination::Pagination,
        patches::{AssetFeaturesPatch, AssetPatch},
        result::{DeleteResult, InsertResult, UpdateResult},
    },
    ops::{AssetFeaturesOps, AssetOps},
    sqlite::SqliteUnit,
};

impl<T> AssetOps for T
where
    T: SqliteUnit,
{
    async fn insert_asset(&mut self, asset: &Asset) -> Result<InsertResult> {
        let res = sqlx::query(
            "
            INSERT INTO assets (
                id, state, media_id,
                created_at, deleted_at,
                title, caption, source_url,
                width, height, accent_color
            )
            VALUES (
                ?, ?, ?,
                ?, ?,
                ?, ?, ?,
                ?, ?, ?
            )
            ",
        )
        .bind(asset.id)
        .bind(asset.state)
        .bind(&asset.media_id)
        .bind(asset.created_at)
        .bind(asset.deleted_at)
        .bind(&asset.title)
        .bind(&asset.caption)
        .bind(&asset.source_url)
        .bind(asset.width)
        .bind(asset.height)
        .bind(asset.accent_color)
        .execute(self.exec())
        .await?;
        Ok(res.into())
    }

    async fn update_asset(&mut self, id: &AssetId, patch: AssetPatch) -> Result<UpdateResult> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE assets
            SET
            ",
        );

        let changes = patch.changes();
        if changes == 0 {
            return Ok(UpdateResult::NoChanges);
        }

        patch.apply_sql(&mut qb);

        qb.push(" WHERE id = ");
        qb.push_bind(id);

        let res = qb.build().execute(self.exec()).await?;
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
        .execute(self.exec())
        .await?;
        Ok(res.into())
    }

    async fn get_assets_bulk(&mut self, ids: &[AssetId]) -> Result<Vec<Asset>> {
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

        let assets = qb.build_query_as().fetch_all(self.exec()).await?;
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

        let assets = qb.build_query_as().fetch_all(self.exec()).await?;
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
        .fetch_all(self.exec())
        .await?;
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
        .fetch_one(self.exec())
        .await?;

        Ok(count)
    }

    async fn get_deleted_assets(&mut self, p: Pagination) -> Result<Vec<Asset>> {
        if p.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT a.* FROM assets AS a
            WHERE a.deleted_at IS NOT null
            ORDER BY a.deleted_at DESC
            ",
        );
        p.apply_sql(&mut qb);

        let assets = qb.build_query_as().fetch_all(self.exec()).await?;
        Ok(assets)
    }
}

impl<T> AssetFeaturesOps for T
where
    T: SqliteUnit,
{
    async fn insert_asset_features(&mut self, af: &AssetFeatures) -> Result<InsertResult> {
        let res = sqlx::query(
            "
            INSERT INTO asset_features (
                asset_id,
                p_hash, a_hash,
                aspect_ratio
            )
            VALUES (
                ?,
                ?, ?,
                ?
            )
            ",
        )
        .bind(af.asset_id)
        .bind(af.p_hash)
        .bind(af.a_hash)
        .bind(af.aspect_ratio)
        .execute(self.exec())
        .await?;
        Ok(res.into())
    }

    async fn update_asset_features(
        &mut self,
        id: &AssetId,
        patch: AssetFeaturesPatch,
    ) -> Result<UpdateResult> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE asset_features
            SET
            ",
        );
        let changes = patch.changes();
        if changes == 0 {
            return Ok(UpdateResult::NoChanges);
        }

        patch.apply_sql(&mut qb);

        qb.push(" WHERE asset_id = ");
        qb.push_bind(id);

        let res = qb.build().execute(self.exec()).await?;
        Ok(res.into())
    }

    async fn get_assets_features_bulk(&mut self, ids: &[AssetId]) -> Result<Vec<AssetFeatures>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT * FROM asset_features
            WHERE asset_id IN
            ",
        );
        qb.push_tuples(ids, |mut qb, id| {
            qb.push_bind(id);
        });

        let assets = qb.build_query_as().fetch_all(self.exec()).await?;
        Ok(assets)
    }
}
