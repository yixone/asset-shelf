use models::{
    entities::{Collection, CollectionAdditions, CollectionAsset},
    types::{CollectionAssetId, CollectionAssetsOrdering, CollectionId, CollectionsOrdering},
};
use result::Result;

use crate::types::{DeleteResult, InsertResult, Pagination, UpdateResult, patch::CollectionPatch};

/// Read operations for data associated with the [`Collection`] domain
pub trait CollectionsReadOps {
    /// Returns [`Collection`] for a given [`CollectionId`]
    async fn get_collection_by_id(&mut self, id: &CollectionId) -> Result<Option<Collection>> {
        self.get_collections_by_ids(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }

    /// Returns a set of [collections](Collection) based on a set of [IDs](CollectionId),
    async fn get_collections_by_ids(&mut self, ids: &[CollectionId]) -> Result<Vec<Collection>>;

    /// Returns a paginated list of collections
    async fn list_collections(
        &mut self,
        p: Pagination,
        o: CollectionsOrdering,
    ) -> Result<Vec<Collection>>;
}

/// Write operations for data associated with the [`Collection`] domain
pub trait CollectionsWriteOps {
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
}

/// Operations for interacting with [`Collection`] relations
pub trait CollectionsRelationsOps {
    /// Returns a set of [collections additions](CollectionAdditions)
    /// based on a set of [IDs](CollectionId),
    async fn get_collections_additions_by_ids(
        &mut self,
        ids: &[CollectionId],
    ) -> Result<Vec<CollectionAdditions>>;

    /// Adds a new asset to the collection by inserting an [`CollectionAsset`] relation
    async fn insert_collection_asset(&mut self, ca: &CollectionAsset) -> Result<InsertResult>;

    /// Remove an asset from the collection by deleting the [`CollectionAsset`] relation
    async fn remove_collection_asset(&mut self, id: &CollectionAssetId) -> Result<DeleteResult>;

    /// Returns a list of [`CollectionAsset`] associations for the specified collection
    async fn get_collection_assets(
        &mut self,
        id: &CollectionId,
        p: Pagination,
        o: CollectionAssetsOrdering,
    ) -> Result<Vec<CollectionAsset>>;
}
