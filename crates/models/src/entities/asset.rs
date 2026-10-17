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

impl Asset {}

impl AssetFeatures {}

impl AssetMeta {}
