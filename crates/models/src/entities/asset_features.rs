use crate::types::AssetId;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct AssetFeatures {
    pub asset_id: AssetId,

    pub p_hash: Option<i64>,
    pub a_hash: Option<i64>,
    pub aspect_ratio: Option<f32>,
}
