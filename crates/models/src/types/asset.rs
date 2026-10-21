/// Represents the lifecycle state of an asset
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
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

impl AssetState {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetState::Pending => "pending",
            AssetState::Processing => "processing",
            AssetState::Ready => "ready",
            AssetState::Failed(..) => "failed",
        }
    }

    pub fn from_persistence(str: &str, failure: Option<AssetFailure>) -> Option<Self> {
        match str {
            "pending" => Some(Self::Pending),
            "processing" => Some(Self::Processing),
            "ready" => Some(Self::Ready),
            "failed" => Some(AssetState::Failed(failure.unwrap_or(AssetFailure::Unknown))),
            _ => None,
        }
    }
}

impl std::fmt::Display for AssetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Describes why processing of an asset failed
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize,),
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
    OriginalFileUnavailable,

    /// Unknown or otherwise unclassified processing error
    ///
    /// See the logs for underlying details
    Unknown,
}

/// The field used for sorting assets
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetSortBy {
    /// Sorting assets by creation date
    CreatedAt,
    /// Sorting assets by deletion date
    DeletedAt,
    /// Sorting assets by original file size
    /// (does not account for the size of derivatives)
    OriginalSize,
}
