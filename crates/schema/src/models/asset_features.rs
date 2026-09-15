//! Asset features model, including its types and behaviour

use crate::id::AssetId;

/// Represents the asset's calculated features
///
/// Asset attributes are a set of parameters used for
/// better display in the web UI and for searching for similar assets
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug)]
pub struct AssetFeatures {
    /// Identifier of the asset to which the features belong
    pub id: AssetId,

    /// Asset width in pixels
    pub width: Option<u32>,

    /// Asset height in pixels
    pub height: Option<u32>,

    /// Average asset hash
    ///
    /// It is calculated based on the deviation of the image colors from the mean value
    pub a_hash: Option<i64>,

    /// Asset perceptual hash
    ///
    /// It is calculated using the image's DCT matrix
    pub p_hash: Option<i64>,
}
