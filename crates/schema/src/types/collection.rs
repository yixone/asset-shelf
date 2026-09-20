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
