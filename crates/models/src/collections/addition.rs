use crate::types::{CollectionId, MediaId};

/// A model containing additional
/// calculated information about the [`Collection`]
#[derive(Debug, Clone)]
pub struct CollectionAdditions {
    /// Identifier of the collection for which the data was calculated
    pub collection: CollectionId,

    /// List of [`MediaId`] for the collection preview
    ///
    /// It is taken from the last `3` saved assets
    pub thumbnails: Vec<MediaId>,

    /// Total number of assets saved in the collection
    pub assets_count: u32,
}
