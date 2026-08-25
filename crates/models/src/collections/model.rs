use chrono::{DateTime, Utc};

use crate::types::CollectionId;

/// Collection domain model
#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Collection {
    /// FlakeID Collection Identifier
    pub id: CollectionId,

    /// Display name of the collection
    pub name: String,
    /// Collection description
    pub description: Option<String>,

    /// Collection creation date
    pub created_at: DateTime<Utc>,
}

impl Collection {
    /// Creates a new [`Collection`]
    pub fn new(id: CollectionId, name: String, description: Option<String>) -> Self {
        Self {
            id,
            name,
            description,
            created_at: Utc::now(),
        }
    }
}
