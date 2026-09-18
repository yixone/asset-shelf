use chrono::{DateTime, Utc};

use crate::{
    id::{AssetId, MediaId},
    types::{AssetState, AssetType},
};

/// Represents an asset managed by the application
///
/// An asset is a primary domain entity for managing user-uploaded data.
/// It contains the identification data, metadata, and lifecycle state
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug)]
pub struct Asset {
    /// Unique asset identifier
    pub id: AssetId,

    /// Identifier of the media associated with the asset
    ///
    /// Represents a 1:1 relation between `Asset` and `Media`
    pub media_id: MediaId,

    /// Optional name for the asset
    pub name: Option<String>,

    /// Optional caption for the asset
    pub caption: Option<String>,

    /// Type of the asset's original media file
    pub asset_type: AssetType,

    /// Current lifecycle state of the asset
    pub state: AssetState,

    /// SHA-1 checksum for the original asset file
    pub sha1: Vec<u8>,

    /// Identifier of the asset that this asset is considered a duplicate of
    pub duplicate_of: Option<AssetId>,

    /// If `true`, the original file has been lost from storage and the asset cannot be used
    pub is_offline: bool,

    /// Asset creation time
    pub created_at: DateTime<Utc>,

    /// Time of the asset's last modification
    pub updated_at: DateTime<Utc>,

    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Asset {
    /// Creates a new pending [`Asset`]
    ///
    /// For fields not specified in the arguments are initialized
    /// with their default values for a pending asset
    pub fn new(
        id: AssetId,
        media: MediaId,
        name: Option<String>,
        caption: Option<String>,
        asset_type: AssetType,
        sha1: Vec<u8>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id,
            media_id: media,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            name,
            caption,
            asset_type,
            state: AssetState::Pending,
            sha1,
            duplicate_of: None,
            is_offline: false,
        }
    }

    /// Returns `true` if the current asset has user-defined metadata
    pub fn has_meta(&self) -> bool {
        self.name.is_some() || self.caption.is_some()
    }

    /// Returns `true` if the current asset is marked as deleted
    pub fn is_soft_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Returns `true` if the current asset is ready for use
    pub fn is_ready(&self) -> bool {
        self.state == AssetState::Ready
    }

    /// Returns `true` if the current asset is a duplicate of another asset
    pub fn is_duplicate(&self) -> bool {
        self.duplicate_of.is_some()
    }
}
