mod bus;
mod channel;
mod event;
mod macros;

pub use bus::EventBus;
pub use channel::EventStream;
pub use event::DynamicEvent;

event! {
    /// New asset creation event
    ///
    /// Called when a new asset is uploaded
    AssetCreatedEvent {
        /// New asset ID
        asset_id: ::models::types::AssetId
    }
}

event! {
    /// Asset deletion event
    ///
    /// Called when the marked asset is permanently deleted
    AssetDeletedEvent {
        /// ID of the deleted asset
        asset: ::models::types::AssetId,
        /// The media ID associated with the deleted asset
        media: ::models::types::MediaId,
    }
}

event! {
    /// The file was found in the database but is missing from the storage
    FileDetachedEvent {
        /// Detached media ID
        media: ::models::types::MediaId,
        /// Detached file path
        path: ::storage::StoragePath,
    }
}
