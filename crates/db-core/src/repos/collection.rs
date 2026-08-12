use models::{
    entities::{Collection, CollectionAsset},
    types::{
        AssetId, CollectionAssetId, CollectionAssetsOrdering, CollectionId, CollectionsOrdering,
    },
};
use result::Result;

use crate::{
    queries::collection::{CollectionItemQuery, CollectionQuery},
    types::{DeleteResult, InsertResult, Pagination, UpdateResult, patch::CollectionPatch},
};

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn insert(&self, collection: &Collection) -> Result<InsertResult>;

    async fn update(
        &self,
        id: CollectionId,
        patch: CollectionPatch,
    ) -> Result<UpdateResult<CollectionQuery>>;

    async fn delete(&self, id: CollectionId) -> Result<DeleteResult>;

    async fn get_items(
        &self,
        id: CollectionId,
        pagination: Pagination,
        order: CollectionAssetsOrdering,
    ) -> Result<Vec<CollectionItemQuery>>;

    async fn add_asset(&self, id: CollectionId, asset: AssetId) -> Result<CollectionAsset>;

    async fn remove_asset(&self, id: CollectionId, rel: CollectionAssetId) -> Result<DeleteResult>;

    async fn get_by_id(&self, id: CollectionId) -> Result<CollectionQuery>;

    async fn list(
        &self,
        pagination: Pagination,
        order: CollectionsOrdering,
    ) -> Result<Vec<CollectionQuery>>;
}
