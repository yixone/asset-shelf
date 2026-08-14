use models::{
    collections::{
        Collection, CollectionAsset,
        view::{CollectionItemView, CollectionView},
    },
    types::{
        AssetId, CollectionAssetId, CollectionAssetsOrdering, CollectionId, CollectionsOrdering,
    },
};
use result::{Result, create_error};

use crate::types::{DeleteResult, InsertResult, Pagination, UpdateResult, patch::CollectionPatch};

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn insert(&self, collection: &Collection) -> Result<InsertResult>;

    async fn update(
        &self,
        id: CollectionId,
        patch: CollectionPatch,
    ) -> Result<UpdateResult<CollectionView>>;

    async fn delete(&self, id: CollectionId) -> Result<DeleteResult>;

    async fn get_items(
        &self,
        id: CollectionId,
        pagination: Pagination,
        order: CollectionAssetsOrdering,
    ) -> Result<Vec<CollectionItemView>>;

    async fn add_asset(
        &self,
        rel: CollectionAssetId,
        id: CollectionId,
        asset: AssetId,
    ) -> Result<CollectionAsset>;

    async fn remove_asset(&self, id: CollectionId, rel: CollectionAssetId) -> Result<DeleteResult>;

    async fn get_by_id(&self, id: CollectionId) -> Result<CollectionView> {
        let q = self.get_by_ids(&[id]).await?;
        q.into_iter().next().ok_or(create_error!(NotFound))
    }

    async fn get_by_ids(&self, ids: &[CollectionId]) -> Result<Vec<CollectionView>>;

    async fn list(
        &self,
        pagination: Pagination,
        order: CollectionsOrdering,
    ) -> Result<Vec<CollectionView>>;
}
