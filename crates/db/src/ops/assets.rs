use models::{
    entities::Asset,
    types::{AssetId, AssetsOrdering},
};
use result::Result;

use crate::types::{DeleteResult, InsertResult, Pagination, UpdateResult, patch::AssetPatch};

/// Read operations for data associated with the [`Asset`] domain
pub trait AssetsReadOps {
    /// Returns [`Asset`] for a given [`AssetId`]
    async fn get_asset_by_id(&mut self, id: &AssetId) -> Result<Option<Asset>> {
        self.get_assets_by_ids(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }

    /// Returns a set of [`assets`](Asset) based on a set of [`IDs`](AssetId)
    async fn get_assets_by_ids(&mut self, ids: &[AssetId]) -> Result<Vec<Asset>>;

    /// Returns a set of assets marked as deleted
    async fn get_deleted_assets(&mut self, p: Pagination, o: AssetsOrdering) -> Result<Vec<Asset>>;

    /// Returns a list of [`assets`](Asset) with [`Pagination`] and filters
    async fn list_assets(&mut self, p: Pagination, o: AssetsOrdering) -> Result<Vec<Asset>>;

    /// Returns a set of random [`assets`](Asset)
    async fn random_assets(&mut self, limit: u32) -> Result<Vec<Asset>>;

    /// Returns the total number of [`assets`](Asset)
    async fn count_assets(&mut self) -> Result<u64>;
}

/// Write operations for data associated with the [`Asset`] domain
pub trait AssetsWriteOps {
    /// Inserts an asset into the database and returns an [`InsertResult`]
    async fn insert_asset(&mut self, a: &Asset) -> Result<InsertResult>;

    /// Updates the [`Asset`] record with the
    /// specified ID according to the provided [`AssetPatch`]
    async fn update_asset(
        &mut self,
        id: &AssetId,
        patch: AssetPatch,
    ) -> Result<UpdateResult<Asset>>;

    /// Removes an [`Asset`] from the database by the specified ID
    async fn delete_asset(&mut self, id: &AssetId) -> Result<DeleteResult>;
}

/// Operations on data associated with the [`Asset`] domain, used for maintenance
///
/// The methods in this set of operations do not interact with user I/O
pub trait AssetsMaintenanceOps {
    /// Returns a set of assets requiring processing
    async fn get_unprocessed_assets(&mut self, limit: u32) -> Result<Vec<Asset>>;
}
