//! Definition of the asset domain, its types, and methods

use crate::{
    id::AssetId,
    types::{TimeUtc, media::MediaType},
};

/// Represents an asset managed by the application
///
/// An asset is a primary domain entity for managing uploaded user data.
/// It contains the identification data, metadata, and lifecycle state
#[derive(Debug)]
pub struct Asset {
    /// Unique asset identifier
    pub id: AssetId,

    /// Asset creation datetime
    pub created_at: TimeUtc,

    /// Datetime of the asset's last modification
    pub updated_at: TimeUtc,

    /// Optional asset deletion datetime
    ///
    /// If `Some`, the asset is considered deleted
    pub deleted_at: Option<TimeUtc>,

    /// Optional name for the asset
    pub name: Option<String>,

    /// Optional caption for the asset
    pub caption: Option<String>,

    /// Type of the asset's original media file
    pub media_type: MediaType,

    /// Asset lifecycle state
    pub state: AssetState,
}

impl Asset {}

/// Asset lifecycle state
#[derive(Debug, Clone, Copy)]
pub enum AssetState {
    /// The asset has been uploaded and is awaiting processing
    Pending,
    /// The asset is being processed
    Processing,
    /// The asset has been processed and is
    /// ready for display and management
    Ready,
    /// Asset processing failed with an error
    Failed,
    /// The media file associated with the asset
    /// was not found in the storage
    Offline,
}
