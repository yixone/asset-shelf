use chrono::{DateTime, Utc};
use mime::MimeKind;

use crate::{
    id::{AssetId, FileGroupId},
    types::{AssetState, Color, PerceptualHash},
};

/// Represents an asset and its current state within the media library
///
/// An asset is a logical media item that may have multiple associated
/// file variants and derived features
#[derive(Debug)]
pub struct Asset {
    /// Unique asset ID
    id: AssetId,

    /// Asset features
    features: AssetFeatures,
    /// User-provided asset metadata
    meta: AssetMeta,

    /// Parrent asset ID
    parent_id: Option<AssetId>,
    /// ID of the asset that this asset is considered a duplicate of
    duplicate_of: Option<AssetId>,

    /// Identifier of the file group associated with the asset
    file_group_id: FileGroupId,
    //// Original asset file name
    original_filename: Option<String>,
    /// Type of the asset's original media file
    original_mime: MimeKind,

    /// Current lifecycle state of the asset
    state: AssetState,
    /// If `true`, the original file has been lost from storage and the asset cannot be used
    is_offline: bool,

    /// Asset creation time
    created_at: DateTime<Utc>,
    /// Time of the asset's last modification
    updated_at: Option<DateTime<Utc>>,
    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    deleted_at: Option<DateTime<Utc>>,
}

/// Derived features calculated from an asset's media
///
/// Features are generated from the asset's media and may be unavailable
/// while media processing has not completed
#[derive(Debug, Default)]
pub struct AssetFeatures {
    /// Asset accent color
    accent_color: Option<Color>,

    /// Asset perceptual hash
    p_hash: Option<PerceptualHash>,
    /// Average asset hash
    a_hash: Option<PerceptualHash>,

    /// Asset width and height in pixels
    dimension: Option<(u32, u32)>,
}

/// User-provided metadata associated with an asset
#[derive(Debug)]
pub struct AssetMeta {
    /// Optional name for the asset
    name: Option<String>,
    /// Optional caption for the asset
    caption: Option<String>,

    /// URL from which the asset was obtained
    source_url: Option<String>,
}

impl Asset {
    /// Returns the ID of this [`Asset`]
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// Returns a reference to the derived features of this [`Asset`]
    pub fn features(&self) -> &AssetFeatures {
        &self.features
    }

    /// Returns a reference to the user-provided metadata of this [`Asset`]
    pub fn meta(&self) -> &AssetMeta {
        &self.meta
    }

    /// Returns the ID of the parent [`Asset`]
    pub fn parent_id(&self) -> Option<AssetId> {
        self.parent_id
    }

    #[inline]
    /// Returns `true` if the [`Asset`] has a parent
    pub fn is_child(&self) -> bool {
        self.parent_id().is_some()
    }

    /// Returns the ID of the asset this [`Asset`] duplicates
    pub fn duplicate_of(&self) -> Option<AssetId> {
        self.duplicate_of
    }

    #[inline]
    /// Returns `true` if this [`Asset`] is marked as a duplicate
    pub fn is_duplicate(&self) -> bool {
        self.duplicate_of().is_some()
    }

    /// Returns the ID of the group of files associated with this asset
    pub fn file_group_id(&self) -> &FileGroupId {
        &self.file_group_id
    }

    /// Returns the name of the original file associated with this [`Asset`]
    pub fn original_filename(&self) -> Option<&str> {
        self.original_filename.as_deref()
    }

    /// Returns the MIME kind of the original file associated with this [`Asset`]
    pub fn original_mime(&self) -> MimeKind {
        self.original_mime
    }

    /// Returns the lifecycle state of this [`Asset`]
    pub fn state(&self) -> AssetState {
        self.state
    }

    /// Returns the creation timestamp of this [`Asset`]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the last update timestamp of this [`Asset`]
    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }

    /// Returns the deletion timestamp of this [`Asset`]
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    #[inline]
    /// Returns `true` if this [`Asset`] is marked as deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }

    /// Returns `true` if this [`Asset`] is marked as offline
    pub fn is_offline(&self) -> bool {
        self.is_offline
    }
}

impl AssetFeatures {
    /// Returns the accent color of this [`AssetFeatures`]
    pub fn accent_color(&self) -> Option<Color> {
        self.accent_color
    }

    /// Returns the aspect ratio of the asset
    ///
    /// Returns `None` if the asset dimensions are unavailable
    pub fn aspect_ratio(&self) -> Option<f32> {
        if let Some((w, h)) = self.dimension {
            Some(w as f32 / h as f32)
        } else {
            None
        }
    }

    /// Returns the pHash of this [`AssetFeatures`]
    pub fn p_hash(&self) -> Option<PerceptualHash> {
        self.p_hash
    }

    /// Returns the aHash of this [`AssetFeatures`]
    pub fn a_hash(&self) -> Option<PerceptualHash> {
        self.a_hash
    }

    /// Returns the asset width in pixels
    pub fn width(&self) -> Option<u32> {
        self.dimension.map(|(w, ..)| w)
    }

    /// Returns the asset height in pixels
    pub fn height(&self) -> Option<u32> {
        self.dimension.map(|(_, h)| h)
    }
}

impl AssetMeta {
    /// Returns the name of this [`AssetMeta`]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the caption of this [`AssetMeta`]
    pub fn caption(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    /// Returns the source url of this [`AssetMeta`]
    pub fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }
}
