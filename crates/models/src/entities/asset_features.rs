use crate::types::{AssetId, Color};

/// Computable features of the asset's media file
///
/// Used for the function of searching for similar assets in the collection
#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct AssetFeatures {
    /// Identifier of the asset to which the feature set is linked
    pub asset_id: AssetId,

    /// Asset perceptual hash
    pub p_hash: Option<i64>,
    /// Asset average hash
    pub a_hash: Option<i64>,

    /// Media width
    pub width: Option<u32>,
    /// Media height
    pub height: Option<u32>,
    /// Media file accent color
    pub accent_color: Option<Color>,
}
