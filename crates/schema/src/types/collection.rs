use crate::id::{CollectionId, MediaId};

/// Summary information about a collection
///
/// Contains the total number of assets and a set of preview media
pub struct CollectionSummary {
    /// ID of the collection for which the data was calculated
    pub id: CollectionId,

    /// Total number of assets in the collection
    pub assets_count: u64,

    /// Collection previews
    pub thumnails: Vec<MediaId>,
}

/// Specifies the field used to sort collections
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[derive(Debug)]
pub enum CollectionSortBy {
    /// Sort by creation date
    CreatedAt,
    /// Sort by last modified date
    UpdatedAt,
    /// Sort by number of assets in the collection
    AssetsCount,
}
