use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::{
    entities::{Asset, AssetFeatures, AssetMeta, File},
    id::AssetId,
    types::{AssetState, Color, PerceptualHash},
};

#[macro_use]
mod _macro;

mod field;
pub use field::PatchField;

patch! {
     /// Partial update for a [`File`]
    FilePatch {
        duration: Option<Duration>
    },
    File
}

patch! {
    /// Partial update for an [`Asset`]
    AssetPatch {
        state: AssetState,
        duplicate_of: Option<AssetId>,
        parent_id: Option<AssetId>,
        deleted_at: Option<DateTime<Utc>>,
        is_offline: bool,
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
        dimension: Option<(u32, u32)>,
        p_hash: Option<PerceptualHash>,
        a_hash: Option<PerceptualHash>,
    },
    AssetFeatures
}
