use db_core::{
    ops::{Operation, create_asset::CreateAssetOp},
    types::InsertResult,
};
use models::entities::{Asset, AssetFeatures, Media, MediaFile};
use result::{Result, error::ResultExt};
use sqlx::SqliteTransaction;

use crate::queries;

pub struct SqliteCreateAssetOp<'a> {
    pub(crate) tx: SqliteTransaction<'a>,
}

#[async_trait::async_trait]
impl<'a> Operation for SqliteCreateAssetOp<'a> {
    async fn commit(self: Box<Self>) -> Result<()> {
        self.tx.commit().await.to_app_err()
    }

    async fn rollback(self: Box<Self>) -> Result<()> {
        self.tx.rollback().await.to_app_err()
    }
}

#[async_trait::async_trait]
impl<'a> CreateAssetOp for SqliteCreateAssetOp<'a> {
    async fn insert_asset(&mut self, asset: &Asset) -> Result<InsertResult> {
        queries::asset::insert_asset(asset, &mut *self.tx).await
    }

    async fn insert_features(&mut self, features: &AssetFeatures) -> Result<InsertResult> {
        queries::asset::insert_asset_features(features, &mut *self.tx).await
    }

    async fn insert_media(&mut self, media: &Media) -> Result<InsertResult> {
        queries::media::insert_media(media, &mut *self.tx).await
    }

    async fn insert_media_file(&mut self, file: &MediaFile) -> Result<InsertResult> {
        queries::media::insert_media_file(file, &mut *self.tx).await
    }
}
