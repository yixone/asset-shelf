use models::{
    entities::{Asset, AssetFeatures},
    types::{AssetId, AssetsOrdering},
};
use result::Result;

use crate::types::{
    DeleteResult, InsertResult, Pagination, UpdateResult,
    patches::{AssetFeaturesPatch, AssetPatch},
};

pub trait AssetOps {
    async fn insert_asset(&mut self, a: &Asset) -> Result<InsertResult>;

    async fn update_asset(&mut self, id: &AssetId, patch: AssetPatch) -> Result<UpdateResult>;
    async fn delete_asset(&mut self, id: &AssetId) -> Result<DeleteResult>;

    async fn get_asset(&mut self, id: &AssetId) -> Result<Option<Asset>> {
        self.get_assets_bulk(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }
    async fn get_assets_bulk(&mut self, ids: &[AssetId]) -> Result<Vec<Asset>>;

    async fn list_assets(&mut self, p: Pagination, o: AssetsOrdering) -> Result<Vec<Asset>>;
    async fn random_assets(&mut self, limit: u32) -> Result<Vec<Asset>>;
    async fn count_assets(&mut self) -> Result<u64>;

    async fn get_unprocessed_assets(&mut self, limit: u32) -> Result<Vec<Asset>>;

    async fn get_deleted_assets(&mut self, p: Pagination) -> Result<Vec<Asset>>;
}

// TODO: add `get similarity search candidates` or something like this
pub trait AssetFeaturesOps {
    async fn insert_asset_features(&mut self, af: &AssetFeatures) -> Result<InsertResult>;

    async fn update_asset_features(
        &mut self,
        id: &AssetId,
        patch: AssetFeaturesPatch,
    ) -> Result<UpdateResult>;

    async fn get_asset_features(&mut self, id: &AssetId) -> Result<Option<AssetFeatures>> {
        self.get_assets_features_bulk(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }
    async fn get_assets_features_bulk(&mut self, ids: &[AssetId]) -> Result<Vec<AssetFeatures>>;
}
