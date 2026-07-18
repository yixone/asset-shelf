use models::{
    entities::{Asset, AssetFeatures},
    types::AssetId,
};

use crate::core::{
    Result,
    pagination::Pagination,
    patches::{AssetFeaturesPatch, AssetPatch},
    result::{DeleteResult, InsertResult, UpdateResult},
};

pub trait AssetOps {
    async fn insert_asset(&mut self, a: &Asset) -> Result<InsertResult>;

    async fn update_asset(&mut self, id: &AssetId, patch: AssetPatch) -> Result<UpdateResult>;
    async fn delete_asset(&mut self, id: &AssetId) -> Result<DeleteResult>;

    async fn get_asset(&mut self, id: &AssetId) -> Result<Option<Asset>>;
    async fn get_many_assets(&mut self, ids: &[AssetId]) -> Result<Vec<Asset>>;

    // TODO: ADD SORTING QUERY!
    async fn list_assets(&mut self, p: Pagination) -> Result<Vec<Asset>>;
    async fn count_assets(&mut self) -> Result<u64>;
}

pub trait AssetFeaturesOps {
    async fn insert_asset_features(&mut self, af: &AssetFeatures) -> Result<InsertResult>;

    async fn update_asset_features(
        &mut self,
        id: &AssetId,
        patch: AssetFeaturesPatch,
    ) -> Result<UpdateResult>;

    async fn get_asset_features(&mut self, id: &AssetId) -> Result<Option<AssetFeatures>>;
    async fn get_many_assets_features(&mut self, ids: &[AssetId]) -> Result<Vec<AssetFeatures>>;

    // TODO: add get similarity search candidates or something like this
}
