use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mime::MimeKind;

use crate::{
    entities::File,
    id::AssetId,
    types::{AssetFileVariant, AssetState, Color, PerceptualHash},
};

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
    /// Returns the width of `Asset`
    pub fn width(&self) -> Option<u32> {
        self.dimension.map(|(w, _)| w)
    }

    /// Returns the height of `Asset`
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
}

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
    pub fn original(&self) -> Option<&File> {
        self.files.get(&AssetFileVariant::Original)
    }

    pub fn thumbnail(&self) -> Option<&File> {
        self.files.get(&AssetFileVariant::Thumbnail)
    }
}

pub struct AssetMeta {
    /// Optional name for the asset
    pub name: Option<String>,

    /// Optional caption for the asset
    pub caption: Option<String>,
}

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
