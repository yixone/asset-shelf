/// Represents the lifecycle state of an asset
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetState {
    /// The asset has been uploaded and is awaiting processing
    Pending,

    /// The asset is being processed
    Processing,

    /// The asset has been processed and is ready for use
    Ready,

    /// Asset processing failed with an error
    Failed,
}
