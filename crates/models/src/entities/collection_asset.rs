use chrono::{DateTime, Utc};

use crate::types::{AssetId, CollectionAssetId, CollectionId};

/// A model that describes the relationship between an asset and a collection
#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct CollectionAsset {
    /// Unique relation identifier
    pub id: CollectionAssetId,
    /// ID of the asset being linked to
    pub asset_id: AssetId,
    /// ID of the collection where the asset is saved
    pub collection_id: CollectionId,
    /// Date the asset was saved to the collection
    pub created_at: DateTime<Utc>,
}
