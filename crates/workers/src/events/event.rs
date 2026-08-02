use models::types::{AssetId, MediaId};
use storage::StoragePath;

events! {
    AssetCreated => AssetCreatedEvent,
    AssetDeleted => AssetDeletedEvent,
    MediaDetched => FileDetachedEvent
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
app_event!(
    /// The file was found in the database but is missing from the storage
    FileDetachedEvent {
        media: MediaId,
        path: StoragePath
    }
);
