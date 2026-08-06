use models::{
    entities::AssetFeatures,
    types::{AssetId, Color},
};
use result::{Result, error::ResultExt};
use sqlx::QueryBuilder;

use crate::{
    ops::{AssetFeaturesReadOps, AssetFeaturesWriteOps},
    sqlite::SqliteExecutor,
    types::{InsertResult, Pagination, UpdateResult, patch::AssetFeaturesPatch},
};

impl<T> AssetFeaturesReadOps for T
where
    T: SqliteExecutor,
{
    async fn get_assets_features_by_ids(&mut self, ids: &[AssetId]) -> Result<Vec<AssetFeatures>> {
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

        let assets = qb
            .build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()?;
        Ok(assets)
    }

    async fn get_asset_features_similarity_candidates(
        &mut self,
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
        .fetch_all(self.executor())
        .await
        .to_app_err()?;

        Ok(candidates)
    }
}

impl<T> AssetFeaturesWriteOps for T
where
    T: SqliteExecutor,
{
    async fn insert_asset_features(&mut self, af: &AssetFeatures) -> Result<InsertResult> {
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
        .bind(af.asset_id)
        .bind(af.p_hash)
        .bind(af.a_hash)
        .bind(af.width)
        .bind(af.height)
        .bind(af.accent_color)
        .execute(self.executor())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn update_asset_features(
        &mut self,
        id: &AssetId,
        patch: AssetFeaturesPatch,
    ) -> Result<UpdateResult<AssetFeatures>> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE asset_features
            SET
            ",
        );

        patch.apply_sql(&mut qb);

        qb.push(" WHERE asset_id = ");
        qb.push_bind(id);

        qb.push(" RETURNING * ");

        let res = qb
            .build_query_as()
            .fetch_optional(self.executor())
            .await
            .to_app_err()?;
        Ok(res.into())
    }
}
