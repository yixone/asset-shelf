use crate::types::{AssetId, Color};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct AssetFeatures {
    pub asset_id: AssetId,

    pub p_hash: Option<i64>,
    pub a_hash: Option<i64>,

    pub width: Option<u32>,
    pub height: Option<u32>,
    pub accent_color: Option<Color>,
}
