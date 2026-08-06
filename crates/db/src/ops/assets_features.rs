use models::{
    entities::AssetFeatures,
    types::{AssetId, Color},
};
use result::Result;

use crate::types::{InsertResult, Pagination, UpdateResult, patch::AssetFeaturesPatch};

/// Read operations for data associated with the [`AssetFeatures`] domain
pub trait AssetFeaturesReadOps {
    /// Returns [`AssetFeatures`] for a given [`AssetId`]
    async fn get_asset_features_by_id(&mut self, id: &AssetId) -> Result<Option<AssetFeatures>> {
        self.get_assets_features_by_ids(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }

    /// Returns a set of [`assets features`](AssetFeatures) based on a set of [`IDs`](AssetId)
    async fn get_assets_features_by_ids(&mut self, ids: &[AssetId]) -> Result<Vec<AssetFeatures>>;

    /// Returns candidates for a search based on similar [`AssetFeatures`]
    async fn get_asset_features_similarity_candidates(
        &mut self,
        color: Color,
        aspect_ratio: f32,
        p: Pagination,
    ) -> Result<Vec<AssetFeatures>>;
}

/// Write operations for data associated with the [`AssetFeatures`] domain
pub trait AssetFeaturesWriteOps {
    /// Inserts an [`AssetFeatures`] into the database and returns an [`InsertResult`]
    async fn insert_asset_features(&mut self, af: &AssetFeatures) -> Result<InsertResult>;

    /// Updates the [`AssetFeatures`] record with the
    /// specified ID according to the provided [`AssetFeaturesPatch`]
    async fn update_asset_features(
        &mut self,
        id: &AssetId,
        patch: AssetFeaturesPatch,
    ) -> Result<UpdateResult<AssetFeatures>>;
}
