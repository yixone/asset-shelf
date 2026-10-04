#[macro_use]
mod _macro;

mod field;
use chrono::{DateTime, Utc};
pub use field::PatchField;

use crate::{
    entities::{
        Asset, File,
        asset::{AssetFeatures, AssetMeta},
    },
    id::AssetId,
    types::{AssetState, Color, PerceptualHash},
};

patch! {
    /// Partial update for a [`File`]
    FilePatch {
        duration_ms: Option<i64>
    },
    File
}

patch! {
    /// Partial update for an [`Asset`]
    AssetPatch {
        state: AssetState,
        deleted_at: Option<DateTime<Utc>>,
        duplicate_of: Option<AssetId>,
        is_offline: bool
    },
    Asset
}

patch! {
    /// Partial update for an [`AssetMeta`]
    AssetMetaPatch {
        name: Option<String>,
        caption: Option<String>,
        source_url: Option<String>
    },
    AssetMeta
}

patch! {
    /// Partial update for an [`AssetFeatures`]
    AssetFeaturesPatch {
        accent_color: Option<Color>,
        p_hash: Option<PerceptualHash>,
        a_hash: Option<PerceptualHash>,
        dimension: Option<(u32, u32)>
    },
    AssetFeatures
}
