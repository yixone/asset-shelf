/// Represents the lifecycle state of an asset
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Specifies the field used to sort assets
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[derive(Debug)]
pub enum AssetSortBy {
    /// Sort by creation date
    CreatedAt,
    /// Sort by last modified date
    UpdatedAt,
    /// Sort by original file size
    FileSize,
}
