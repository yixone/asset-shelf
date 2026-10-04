use crate::{
    entities::{Asset, FileGroup},
    id::AssetId,
    patches::{AssetFeaturesPatch, AssetMetaPatch, AssetPatch},
    ports::UnitOfWork,
    result::Result,
};

/// Provides read and write access to [`Asset`] data
///
/// Implementations are responsible for loading assets and applying changes
/// to their persisted state
#[async_trait::async_trait]
pub trait AssetDatabase {
    /// Loads an [`Asset`] together with its associated [`FileGroup`]
    ///
    /// Returns `None` if the asset does not exist
    async fn get_asset(&self, id: AssetId) -> Result<Option<(Asset, FileGroup)>>;

    /// Updates the specified [`Asset`] using the given patch
    ///
    /// Returns `true` if the asset was updated, or `false` if it does not exist
    async fn update_asset(&self, id: AssetId, patch: &AssetPatch) -> Result<bool>;

    /// Updates the user-provided metadata of the specified [`Asset`]
    ///
    /// Returns `true` if the asset was updated, or `false` if it does not exist
    async fn update_asset_meta(&self, id: AssetId, patch: &AssetMetaPatch) -> Result<bool>;

    /// Updates the derived features of the specified [`Asset`]
    ///
    /// Returns `true` if the asset was updated, or `false` if it does not exist
    async fn update_asset_features(&self, id: AssetId, patch: &AssetFeaturesPatch) -> Result<bool>;
}

/// Provides operations for creating persisted [`Asset`]s
#[async_trait::async_trait]
pub trait AssetWriter: UnitOfWork {
    /// Persists a new [`Asset`]
    async fn insert_asset(&mut self, asset: &Asset) -> Result<()>;
}
