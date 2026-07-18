use crate::AssetId;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct AssetFeatures {
    pub asset_id: AssetId,

    pub p_hash: i64,
    pub a_hash: i64,
    pub aspect_ratio: f32,
}
