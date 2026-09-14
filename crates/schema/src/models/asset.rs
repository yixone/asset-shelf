//! Asset domain model, including its types and behavour

use chrono::{DateTime, Utc};

use crate::{
    id::{AssetId, MediaId},
    types::media::MediaType,
};

/// Represents an asset managed by the application
///
/// An asset is a primary domain entity for managing user-uploaded data.
/// It contains the identification data, metadata, and lifecycle state
#[derive(Debug)]
pub struct Asset {
    /// Unique asset identifier
    pub id: AssetId,

    /// Identifier of the media related with the asset
    ///
    /// Reflects a 1:1 relationship between [`Asset`] : [`Media`]
    pub media: MediaId,

    /// Asset creation datetime
    pub created_at: DateTime<Utc>,

    /// Datetime of the asset's last modification
    pub updated_at: DateTime<Utc>,

    /// Optional asset deletion datetime
    ///
    /// If `Some`, the asset is considered deleted
    pub deleted_at: Option<DateTime<Utc>>,

    /// Optional name for the asset
    pub name: Option<String>,

    /// Optional caption for the asset
    pub caption: Option<String>,

    /// Type of the asset's original media file
    pub media_type: MediaType,

    /// Current lifecycle state of the asset
    pub state: AssetState,
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
        media_type: MediaType,
    ) -> Self {
        Self {
            id,
            media,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            name,
            caption,
            media_type,
            state: AssetState::Pending,
        }
    }

    /// Returns `true` if the current asset has user-defined metadata
    pub fn has_meta(&self) -> bool {
        self.name.is_some() || self.caption.is_some()
    }

    /// Returns `true` if the current asset is marked as deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Represents the current lifecycle state of an asset
#[derive(Debug, Clone, Copy)]
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
