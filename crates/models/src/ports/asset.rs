use crate::{
    Result,
    entities::{Asset, AssetFeatures, AssetMeta, FileGroup},
    id::AssetId,
    patches,
    types::{AssetSortBy, DeletedVisibility, Pagination, SortOrder},
};

#[async_trait::async_trait]
pub trait AssetDatabase {
    /// Persists a new [`Asset`]
    async fn insert_asset(&mut self, asset: &Asset) -> Result<()>;

    /// Loads an [`Asset`] together with its associated [`FileGroup`]
    ///
    /// Returns `None` if the asset does not exist
    async fn get_asset(&mut self, id: AssetId) -> Result<Option<(Asset, FileGroup)>>;

    /// Lists assets matching the given pagination, ordering and deletion visibility
    async fn list_assets(
        &mut self,
        pagination: Pagination,
        order: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
    ) -> Result<Vec<(Asset, FileGroup)>>;

    /// Updates the specified [`Asset`] using the given patch
    async fn update_asset(&mut self, id: AssetId, patch: &patches::AssetPatch) -> Result<Asset>;

    /// Updates the user-provided metadata of the specified [`Asset`]
    async fn update_asset_meta(
        &mut self,
        id: AssetId,
        patch: &patches::AssetMetaPatch,
    ) -> Result<AssetMeta>;

    /// Updates the derived features of the specified [`Asset`]
    async fn update_asset_features(
        &mut self,
        id: AssetId,
        patch: &patches::AssetFeaturesPatch,
    ) -> Result<AssetFeatures>;

    /// Replace references to one asset with references to another
    ///
    /// - `from` identifies the asset whose references are to be replaced;
    /// - `to` identifies the asset that will be referenced instead
    ///
    /// Returns the number of references replaced
    async fn replace_asset_refs(&mut self, from: AssetId, to: AssetId) -> Result<u32>;

    /// Deletes an [`Asset`] by its identifier
    async fn delete_asset(&mut self, id: AssetId) -> Result<bool>;
}
