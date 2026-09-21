use crate::{
    id::{AssetId, CollectionId},
    types::AssetState,
};

/// New asset creation event
///
/// Triggered when a new asset is uploaded
#[derive(Debug)]
pub struct AssetCreatedEvent {
    pub id: AssetId,
}

/// Asset processing completion event
///
/// Triggered when background asset processing
/// is complete and the asset is ready
#[derive(Debug)]
pub struct AssetReadyEvent {
    pub id: AssetId,
    pub state: AssetState,
}

/// Asset deletion event
///
/// Triggered upon the soft or hard deletion of an asset
#[derive(Debug)]
pub struct AssetDeletedEvent {
    pub is_soft: bool,
    pub id: AssetId,
}

/// Asset restoration event
///
/// Triggered when a soft-deleted asset is restored
pub struct AssetRestoreEvent {
    pub id: AssetId,
}

/// Asset set-offline event
///
/// Triggered when the original asset exists in the database
/// but is missing from the file storage
#[derive(Debug)]
pub struct AssetOfflineEvent {
    pub id: AssetId,
}

/// Event of adding an asset to the collection
#[derive(Debug)]
pub struct CollectionItemAddEvent {
    pub asset: AssetId,
    pub collection: CollectionId,
}

/// Asset removal event from a collection
#[derive(Debug)]
pub struct CollectionItemRemoveEvent {
    pub asset: AssetId,
    pub collection: CollectionId,
}
