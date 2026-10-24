use models::{
    entities::{Asset, FileGroup},
    id::AssetId,
    patches::AssetPatch,
    types::{AssetSortBy, Sha256Checksum},
};

use crate::{
    Result,
    ports::{DeleteFn, UpdateFn},
    types::{CursorPagination, DeletedVisibility, SortOrder, tuples::AssetFull},
};

/// Database operations for assets
#[async_trait::async_trait]
pub trait AssetsDatabase: Send + Sync {
    /// Inserts an [`Asset`] and its associated [`FileGroup`]
    ///
    /// Implementation must preserve the consistency of the inserted data
    async fn insert(&self, asset: &Asset, file_group: &FileGroup) -> Result<()>;

    /// Retrieves an [`Asset`] and its associated data by ID
    ///
    /// Returns `None` if the asset does not exist
    async fn get_by_id(&self, id: AssetId) -> Result<Option<AssetFull>> {
        self.get_many_by_ids(&[id])
            .await
            .map(|a| a.into_iter().next())
    }

    /// Retrieves [`assets`](Asset) and its associated data by IDs
    ///
    /// Missing assets are omitted from the result
    async fn get_many_by_ids(&self, ids: &[AssetId]) -> Result<Vec<AssetFull>>;

    /// Retrieves an [`Asset`] by its SHA-256 checksum
    ///
    /// Used to identify duplicates during the upload stage
    async fn get_by_checksum(&self, checksum: &Sha256Checksum) -> Result<Option<Asset>>;

    /// Retrieves a page of assets using cursor-based pagination and the specified ordering
    ///
    /// Returns the matching assets with their associated files
    async fn list(
        &self,
        pagination: CursorPagination<AssetId>,
        ordering: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetFull>>;

    /// Retrieves a randomly selected [`Asset`] and its associated data
    ///
    /// Returns `None` if no asset can be selected
    async fn get_random(&self) -> Result<Option<AssetFull>>;

    /// Returns the total number of assets
    async fn count_total(&self, deleted: DeletedVisibility) -> Result<u64>;

    /// Applies a patch to an [`Asset`] and persists the result
    ///
    /// Domain invariants enforced by [`Asset::apply_patch`] are preserved.
    async fn update(&self, id: AssetId, patch: AssetPatch) -> Result<Asset> {
        self.update_fn_boxed(id, Box::new(|a| a.apply_patch(patch)))
            .await
    }

    /// Loads an [`Asset`], applies a callback and persists the modified asset
    ///
    /// The callback is executed against the current asset state as part of the
    /// same atomic operation as persistence. If the callback returns an
    /// error, the update is aborted and no changes are persisted
    ///
    /// Implementation should use appropriate database locking or isolation to
    /// prevent conflicting concurrent updates
    ///
    /// Returns the updated asset, or an error if the asset does not exist,
    /// validation fails, or persistence fails
    async fn update_fn_boxed(&self, id: AssetId, f: Box<UpdateFn<Asset>>) -> Result<Asset>;

    /// Deletes an [`Asset`]
    async fn delete(&self, id: AssetId) -> Result<()> {
        self.delete_fn_boxed(id, Box::new(|_| Ok(()))).await
    }

    /// Loads an [`Asset`], validates whether it can be deleted, and deletes it
    ///
    /// The callback and deletion are performed as part of the same atomic operation.
    /// If the callback returns an error, the deletion is aborted
    ///
    /// Returns an error if the asset does not exist, validation fails,
    /// or deletion fails
    async fn delete_fn_boxed(&self, id: AssetId, f: Box<DeleteFn<Asset>>) -> Result<()>;
}

#[async_trait::async_trait]
pub trait AssetsDatabaseExt: AssetsDatabase {
    /// Updates the [`Asset`] using an [`UpdateFn`] callback
    ///
    /// See [`AssetsDatabase::update_fn_boxed`] for the operation's guarantees
    async fn update_fn<Fn>(&self, id: AssetId, f: Fn) -> Result<Asset>
    where
        Fn: FnOnce(&mut Asset) -> std::result::Result<(), models::EntityError> + Send + 'static,
    {
        self.update_fn_boxed(id, Box::new(f)).await
    }

    /// Deletes an [`Asset`] after validating it with an [`DeleteFn`] callback
    ///
    /// See [`AssetsDatabase::delete_fn_boxed`] for the operation's guarantees
    async fn delete_fn<Fn>(&self, id: AssetId, f: Fn) -> Result<()>
    where
        Fn: FnOnce(&Asset) -> std::result::Result<(), models::EntityError> + Send + 'static,
    {
        self.delete_fn_boxed(id, Box::new(f)).await
    }
}

impl<T> AssetsDatabaseExt for T where T: AssetsDatabase {}
