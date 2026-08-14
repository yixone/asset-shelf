use models::{
    assets::{AssetFeatures, AssetState, view::AssetView},
    types::{AssetId, AssetsOrdering, Color},
};
use result::{Result, create_error};

use crate::{
    ops::create_asset::CreateAssetOp,
    types::{
        DeleteResult, Pagination, UpdateResult,
        patch::{AssetFeaturesPatch, AssetPatch},
    },
};

/// Repository for working with the [`Asset`] domain model and its relations
#[async_trait::async_trait]
pub trait AssetRepository: Send + Sync {
    async fn create_op<'a>(&'a self) -> Result<Box<dyn CreateAssetOp + 'a>>;

    async fn update(&self, id: AssetId, patch: AssetPatch) -> Result<UpdateResult<AssetView>>;

    async fn update_state(
        &self,
        id: AssetId,
        state: AssetState,
    ) -> Result<UpdateResult<AssetView>> {
        self.update(id, AssetPatch::new().state(state)).await
    }

    async fn update_features(
        &self,
        id: AssetId,
        patch: AssetFeaturesPatch,
    ) -> Result<UpdateResult<AssetView>>;

    async fn delete(&self, id: AssetId) -> Result<DeleteResult>;

    async fn get_by_id(&self, id: AssetId) -> Result<AssetView> {
        let q = self.get_by_ids(&[id]).await?;
        q.into_iter().next().ok_or(create_error!(NotFound))
    }

    async fn get_by_ids(&self, ids: &[AssetId]) -> Result<Vec<AssetView>>;

    async fn get_deleted(
        &self,
        pagination: Pagination,
        order: AssetsOrdering,
    ) -> Result<Vec<AssetView>>;

    async fn get_random(&self) -> Result<AssetView>;

    async fn get_for_processing(&self, limit: u32) -> Result<Vec<AssetView>>;

    async fn get_for_similar_search(
        &self,
        color: Color,
        aspect_ratio: f32,
        p: Pagination,
    ) -> Result<Vec<AssetFeatures>>;

    async fn count_total(&mut self) -> Result<u64>;

    async fn list(&self, pagination: Pagination, order: AssetsOrdering) -> Result<Vec<AssetView>>;
}
