use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mime::MimeKind;

use crate::{
    entities::File,
    id::AssetId,
    types::{AssetFileVariant, AssetState, Color, PerceptualHash},
};

/// Represents an asset and its current state within the media library
///
/// An asset is a logical media item that may have multiple associated
/// file variants and derived features
///
/// `Asset` is an immutable domain model. Changes to an asset are performed
/// through repository operations rather than by mutating the model directly
pub struct Asset {
    /// Unique asset identifier
    pub id: AssetId,

    /// Current lifecycle state of the asset
    pub state: AssetState,

    /// Asset creation time
    pub created_at: DateTime<Utc>,

    /// Time of the asset's last modification
    pub updated_at: DateTime<Utc>,

    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    pub deleted_at: Option<DateTime<Utc>>,

    /// Asset features
    pub features: AssetFeatures,
    /// Information about media associated with the asset
    pub media: AssetMedia,

    /// Identifier of the asset that this asset is considered a duplicate of
    pub duplicate_of: Option<AssetId>,

    /// Asset metadata
    pub meta: AssetMeta,
}

impl Asset {
    /// Returns `true` if the asset has been marked as deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Returns `true` if the asset is not marked as deleted
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// Returns `true` if the asset is marked as a duplicate of another asset
    pub fn is_duplicate(&self) -> bool {
        self.duplicate_of.is_some()
    }

    /// Returns `true` if the asset has all data
    /// required for use
    pub fn is_available(&self) -> bool {
        !self.is_deleted() && !self.media.is_offline
    }
}

/// Derived features calculated from an asset's media
///
/// Features are generated from the asset's media and may be unavailable
/// while media processing has not completed
pub struct AssetFeatures {
    /// Asset accent color
    pub accent_color: Option<Color>,

    /// Asset perceptual hash
    ///
    /// Calculated using the image's DCT matrix
    pub p_hash: Option<PerceptualHash>,

    /// Average asset hash
    ///
    /// Calculated based on the deviation of the image colors from the mean value
    pub a_hash: Option<PerceptualHash>,

    /// Asset width and height in pixels
    pub dimension: Option<(u32, u32)>,
}

impl AssetFeatures {
    /// Returns the asset width in pixels
    pub fn width(&self) -> Option<u32> {
        self.dimension.map(|(w, _)| w)
    }

    /// Returns the asset height in pixels
    pub fn height(&self) -> Option<u32> {
        self.dimension.map(|(_, h)| h)
    }

    /// Returns the asset aspect ratio
    pub fn aspect_ratio(&self) -> Option<f32> {
        if let Some((w, h)) = self.dimension {
            Some(w as f32 / h as f32)
        } else {
            None
        }
    }

    /// Returns `true` if all required derived
    /// features have been calculated
    pub fn enough_fields(&self) -> bool {
        self.accent_color.is_some()
            && self.p_hash.is_some()
            && self.a_hash.is_some()
            && self.dimension.is_some()
    }
}

/// Information about the media files associated with an asset
///
/// Contains the available file variants together with metadata describing
/// the asset's original media and its storage availability
pub struct AssetMedia {
    /// Files associated with the asset
    pub files: HashMap<AssetFileVariant, File>,

    /// If `true`, the original file has been lost from storage and the asset cannot be used
    pub is_offline: bool,

    /// File name of the asset's original media file
    pub original_file_name: Option<String>,

    /// Type of the asset's original media file
    pub original_mime: MimeKind,
}

impl AssetMedia {
    /// Returns the file associated with the specified variant
    pub fn file(&self, variant: AssetFileVariant) -> Option<&File> {
        self.files.get(&variant)
    }

    /// Returns `true` if the specified file variant is present
    pub fn has_file(&self, variant: AssetFileVariant) -> bool {
        self.files.contains_key(&variant)
    }

    /// Returns the original media file
    pub fn original(&self) -> Option<&File> {
        self.files.get(&AssetFileVariant::Original)
    }

    /// Returns the thumbnail media file
    pub fn thumbnail(&self) -> Option<&File> {
        self.files.get(&AssetFileVariant::Thumbnail)
    }
}

/// User-provided metadata associated with an asset
pub struct AssetMeta {
    /// Optional name for the asset
    pub name: Option<String>,

    /// Optional caption for the asset
    pub caption: Option<String>,
}

/// A lightweight representation of an asset containing only its core state
pub struct AssetDehydrated {
    /// Unique asset identifier
    pub id: AssetId,

    /// Current lifecycle state of the asset
    pub state: AssetState,

    /// Asset creation time
    pub created_at: DateTime<Utc>,

    /// Time of the asset's last modification
    pub updated_at: DateTime<Utc>,

    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    pub deleted_at: Option<DateTime<Utc>>,
}
