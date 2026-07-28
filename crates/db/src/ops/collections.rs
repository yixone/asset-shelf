use models::{
    entities::{Collection, CollectionAdditions, CollectionAsset},
    types::{AssetsOrdering, CollectionAssetId, CollectionId},
};
use result::Result;

use crate::types::{
    DeleteResult, InsertResult, Pagination, UpdateResult, patches::CollectionPatch,
};

/// Set of operations for an abstract [`Collection`] domain repository
///
/// The trait contains all the basic operations for working with collections.
/// To interact with collection entities, use [`CollectionAssetsOps`]
pub trait CollectionsOps {
    /// Inserts the provided [`Collection`] into the database
    /// and returns an [`InsertResult`]
    async fn insert_collection(&mut self, c: &Collection) -> Result<InsertResult>;

    /// Updates the specified [`Collection`] in the database
    /// using the provided [`CollectionPatch`]
    async fn update_collection(
        &mut self,
        id: CollectionId,
        patch: CollectionPatch,
    ) -> Result<UpdateResult<Collection>>;

    /// Removes the specified [`Collection`] from the database
    /// and returns a [`DeleteResult`]
    async fn delete_collection(&mut self, id: CollectionId) -> Result<DeleteResult>;

    /// Returns a paginated list of collections
    async fn list_collections(&mut self, p: Pagination) -> Result<Vec<Collection>>;

    /// Returns the [`Collection`] by [`CollectionId`],
    /// or returns [`None`] if the collection is not in the database
    ///
    /// To avoid the N+1 problem, use the [`CollectionsOps::get_collections_bulk`]
    async fn get_collection(&mut self, id: CollectionId) -> Result<Option<Collection>> {
        self.get_collections_bulk(&[id])
            .await
            .map(|a| a.into_iter().next())
    }

    /// Returns a set of [collections](Collection) based on a set of [IDs](CollectionId),
    /// performing fetching in a single query, which avoids the N+1 problem
    async fn get_collections_bulk(&mut self, ids: &[CollectionId]) -> Result<Vec<Collection>>;

    /// Returns [`CollectionAdditions`] that is calculated in the query
    ///
    /// To avoid the N+1 problem, use the [`CollectionsOps::get_collections_additions_bulk`]
    async fn get_collection_additions(
        &mut self,
        id: CollectionId,
    ) -> Result<Option<CollectionAdditions>> {
        self.get_collections_additions_bulk(&[id])
            .await
            .map(|a| a.into_iter().next())
    }

    /// Returns a set of [collections additions](CollectionAdditions)
    /// based on a set of [IDs](CollectionId),
    async fn get_collections_additions_bulk(
        &mut self,
        ids: &[CollectionId],
    ) -> Result<Vec<CollectionAdditions>>;
}

/// Set of operations for interacting with [`CollectionAsset`]
pub trait CollectionAssetsOps {
    async fn insert_collection_asset(&mut self, ca: &CollectionAsset) -> Result<InsertResult>;

    async fn remove_collection_asset(&mut self, id: &CollectionAssetId) -> Result<DeleteResult>;

    async fn get_collection_assets(
        &mut self,
        id: &CollectionId,
        p: Pagination,
        o: AssetsOrdering,
    ) -> Result<Vec<CollectionAsset>>;
}
