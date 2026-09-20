use chrono::{DateTime, Utc};

use crate::id::{CollectionAssetId, CollectionId};

/// Represents a collection containing assets and sub-collections
///
/// A collection groups related assets and collections into a user-defined sets.
/// Collection assets entries are defined by the `CollectionAsset` table
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug)]
pub struct Collection {
    /// Unique collection identifier
    pub id: CollectionId,

    /// Parent collection ID
    ///
    /// If `None`, the collection is not a sub-collection
    pub parent: Option<CollectionId>,

    /// Display name of the collection
    pub name: String,

    /// Collection description
    pub description: Option<String>,

    /// Collection creation time
    pub created_at: DateTime<Utc>,

    /// Optional asset deletion time
    ///
    /// If `Some`, the collection is considered deleted
    pub deleted_at: Option<DateTime<Utc>>,

    /// Identifier of the relation set as the collection preview
    pub preview_rel_id: Option<CollectionAssetId>,
}

impl Collection {
    /// Creates a new empty [`Collection`]
    pub fn new(
        id: CollectionId,
        parent: Option<CollectionId>,
        name: String,
        description: Option<String>,
    ) -> Collection {
        Collection {
            id,
            parent,
            name,
            description,
            created_at: Utc::now(),
            deleted_at: None,
            preview_rel_id: None,
        }
    }

    /// Returns `true` if the current collection is a child of another collection
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    /// Returns `true` if the current collection is marked as deleted
    pub fn is_soft_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
