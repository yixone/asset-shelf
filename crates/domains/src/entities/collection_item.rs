use chrono::{DateTime, Utc};

use crate::{AssetId, CollectionId, CollectionItemId};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct CollectionItem {
    pub id: CollectionItemId,

    pub asset_id: AssetId,
    pub collection_id: CollectionId,

    pub created_at: DateTime<Utc>,
}
