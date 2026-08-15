use crate::types::{AssetId, Color};

/// Computable features of the asset's media file
///
/// Used for the function of searching for similar assets in the collection
#[derive(Debug, Clone, PartialEq)]
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

impl AssetFeatures {
    /// Creates a new [`AssetFeatures`]
    pub fn new(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            p_hash: None,
            a_hash: None,
            width: None,
            height: None,
            accent_color: None,
        }
    }

    pub const SIMILARITY_COLOR_SHIFT: u8 = 45;
    pub const SIMILARITY_ASPECT_SHIFT: f32 = 0.5;

    /// Returns `True` if `rhs` is a suitable candidate for finding assets similar to `self`
    pub fn is_similar_candidate_for(&self, rhs: &AssetFeatures) -> bool {
        fn resolve_color(f: &AssetFeatures) -> Option<[u8; 3]> {
            let (r, g, b) = f.accent_color.map(|c| c.rgb())?;
            Some([r, g, b])
        }

        fn resolve_aspect(f: &AssetFeatures) -> Option<f32> {
            let (Some(w), Some(h)) = (f.width, f.height) else {
                return None;
            };
            Some(w as f32 / h as f32)
        }

        let (Some(color), Some(rhs_color)) = (resolve_color(self), resolve_color(rhs)) else {
            return false;
        };

        if color
            .into_iter()
            .zip(rhs_color)
            .any(|(s, f)| s.abs_diff(f) <= Self::SIMILARITY_COLOR_SHIFT)
        {
            return true;
        }

        let (Some(aspect), Some(rhs_aspect)) = (resolve_aspect(self), resolve_aspect(rhs)) else {
            return false;
        };

        if (aspect - rhs_aspect).abs() <= Self::SIMILARITY_ASPECT_SHIFT {
            return true;
        }

        false
    }

    /// Returns `true` if all required optional fields are present.
    /// Otherwise, returns `false`
    pub fn enough_fields(&self) -> bool {
        if self.a_hash.is_none()
            || self.accent_color.is_none()
            || self.p_hash.is_none()
            || self.width.is_none()
            || self.height.is_none()
        {
            return false;
        }

        true
    }
}
