use chrono::{DateTime, Utc};
use mimetype::MimeKind;

use crate::types::{AssetId, MediaId};

/// An asset domain representing a media file in storage
#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Asset {
    /// Unique numeric asset ID
    pub id: AssetId,
    /// Asset state
    pub state: AssetState,

    /// Related media group identifier
    pub media_id: MediaId,
    /// Generalized asset media type
    pub media_type: MimeKind,

    /// Asset creation date
    pub created_at: DateTime<Utc>,
    /// Asset deletion date
    pub deleted_at: Option<DateTime<Utc>>,

    /// Asset title
    pub title: Option<String>,
    /// Optional asset description
    pub caption: Option<String>,
    /// URL from which the asset was obtained
    pub source_url: Option<String>,
}

/// Asset state within its lifecycle
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum AssetState {
    /// The asset has only been uploaded
    /// and has not yet been processed
    Pending,
    /// The asset is processed by a background worker
    Processing,
    /// The asset has been processed and is ready for display
    Ready,
    /// An error occurred while working with the asset
    Failed,
}
