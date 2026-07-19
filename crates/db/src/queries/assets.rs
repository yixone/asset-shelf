use models::{
    entities::{Asset, AssetFeatures},
    types::AssetId,
};

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
            r#"
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
            "#,
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
        todo!()
    }
    async fn delete_asset(&mut self, id: &AssetId) -> Result<DeleteResult> {
        todo!()
    }

    async fn get_asset(&mut self, id: &AssetId) -> Result<Option<Asset>> {
        todo!()
    }
    async fn get_assets_bulk(&mut self, ids: &[AssetId]) -> Result<Vec<Asset>> {
        todo!()
    }

    async fn list_assets(&mut self, p: Pagination) -> Result<Vec<Asset>> {
        todo!()
    }
    async fn random_assets(&mut self, limit: u32) -> Result<Vec<Asset>> {
        todo!()
    }
    async fn count_assets(&mut self) -> Result<u64> {
        todo!()
    }
}

impl<T> AssetFeaturesOps for T
where
    T: SqliteUnit,
{
    async fn insert_asset_features(&mut self, af: &AssetFeatures) -> Result<InsertResult> {
        todo!()
    }

    async fn update_asset_features(
        &mut self,
        id: &AssetId,
        patch: AssetFeaturesPatch,
    ) -> Result<UpdateResult> {
        todo!()
    }

    async fn get_asset_features(&mut self, id: &AssetId) -> Result<Option<AssetFeatures>> {
        todo!()
    }

    async fn get_assets_features_bulk(&mut self, ids: &[AssetId]) -> Result<Vec<AssetFeatures>> {
        todo!()
    }
}
