/// Asset state within its lifecycle
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum AssetState {
    /// The asset has only been uploaded
    /// and has not yet been processed
    Pending,
    /// The asset is processed by a background worker
    Processing,
    /// The asset has been processed and is ready for display
    Ready,
    /// An error occurred while working with the asset
    Failed,
}
