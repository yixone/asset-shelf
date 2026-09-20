use chrono::{DateTime, Utc};

use crate::id::{AssetId, CollectionAssetId, CollectionId};

/// Represents the `asset`:`collection` relationship
///
/// A relational table is used to establish an N:1
/// relationship between an asset and a collection
pub struct CollectionAsset {
    /// Unique relation identifier
    pub id: CollectionAssetId,

    /// Relation creation time
    pub created_at: DateTime<Utc>,

    /// Identifier of the collection to which the item was added
    pub collection: CollectionId,

    /// The identifier of the asset added to the collection
    pub asset: AssetId,
}

impl CollectionAsset {
    /// Creates a new [`CollectionAsset`]
    pub fn new(id: CollectionAssetId, collection: CollectionId, asset: AssetId) -> Self {
        Self {
            id,
            created_at: Utc::now(),
            collection,
            asset,
        }
    }
}
