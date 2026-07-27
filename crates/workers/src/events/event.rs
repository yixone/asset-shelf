use models::types::{AssetId, MediaId};

events! {
    AssetCreated => AssetCreatedEvent,
    AssetDeleted => AssetDeletedEvent
}

app_event!(
    /// New asset creation event
    ///
    /// Called when a new asset is uploaded
    AssetCreatedEvent {
        /// New asset ID
        asset: AssetId
    }
);
app_event!(
    /// Asset deletion event
    ///
    /// Called when the marked asset is permanently deleted
    AssetDeletedEvent {
        /// ID of the deleted asset
        asset: AssetId,
        /// The media ID associated with the deleted asset
        media: MediaId
    }
);
