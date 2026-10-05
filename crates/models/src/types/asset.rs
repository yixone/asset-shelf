/// Represents the lifecycle state of an asset
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", tag = "state", content = "state_details")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetState {
    /// The asset has been created and is awaiting processing
    Pending,

    /// The asset is being processed
    Processing,

    /// The asset has been processed and is ready for use
    Ready,

    /// Processing of the asset failed
    Failed(AssetFailure),
}

/// Describes why processing of an asset failed
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetFailure {
    /// The asset has invalid or unsupported dimensions
    InvalidDimension,

    /// The asset has an invalid or unsupported duration
    InvalidDuration,

    /// Media metadata could not be read or parsed
    ProbeFailed,

    /// The asset's original file is unavailable in storage
    FileOffline,

    /// Unknown or otherwise unclassified processing error
    ///
    /// See the logs for underlying details
    Unknown,
}
