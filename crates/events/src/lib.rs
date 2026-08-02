use models::types::{AssetId, MediaId};
use storage::StoragePath;

#[macro_use]
mod macros;
mod bus;
mod channel;
mod routing;

pub use bus::EventBus;
pub use channel::{EventSender, EventStream};
pub use routing::AbstractEvent;
use routing::EventRoutingError;

/// New asset creation event
///
/// Called when a new asset is uploaded
#[derive(Debug, Clone)]
pub struct AssetCreatedEvent {
    /// New asset ID
    pub asset: AssetId,
}

/// Asset deletion event
///
/// Called when the marked asset is permanently deleted
#[derive(Debug, Clone)]
pub struct AssetDeletedEvent {
    /// ID of the deleted asset
    pub asset: AssetId,
    /// The media ID associated with the deleted asset
    pub media: MediaId,
}

/// The file was found in the database but is missing from the storage
#[derive(Debug, Clone)]
pub struct FileDetachedEvent {
    pub media: MediaId,
    pub path: StoragePath,
}

events! {
    AssetCreated => AssetCreatedEvent,
    AssetDeleted => AssetDeletedEvent,
    MediaDetched => FileDetachedEvent
}
