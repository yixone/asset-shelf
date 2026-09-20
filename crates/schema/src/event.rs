use crate::{
    id::{AssetId, CollectionId},
    types::AssetState,
};

pub struct AssetCreatedEvent {
    pub id: AssetId,
}

pub struct AssetReadyEvent {
    pub id: AssetId,
    pub state: AssetState,
}

pub struct AssetDeletedEvent {
    pub id: AssetId,
}

pub struct AssetOfflineEvent {
    pub id: AssetId,
}

pub struct CollectionItemAddEvent {
    pub asset: AssetId,
    pub collection: CollectionId,
}

pub struct CollectionItemRemoveEvent {
    pub asset: AssetId,
    pub collection: CollectionId,
}
