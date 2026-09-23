use chrono::{DateTime, Utc};
use mime::MimeType;

use crate::{
    id::{MediaFileId, MediaId},
    types::{MediaFileKey, MediaVariant},
};

/// Represents a stored file associated with a `Media` object
///
/// Contains the file's storage location, variant, MIME type, size
/// and optional duration for time-based media
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug)]
pub struct MediaFile {
    /// Media file identifier
    pub id: MediaFileId,
    /// Identifier of the media with which the file is associated
    pub media_id: MediaId,

    /// File creation time
    pub created_at: DateTime<Utc>,

    /// Variant of this file
    pub variant: MediaVariant,
    /// File blob key in application storage
    pub file_key: MediaFileKey,

    /// File size in bytes
    pub size_bytes: i64,
    /// File MIME type
    pub mime_type: MimeType,

    /// File duration in milliseconds (for supported files)
    pub duration_ms: Option<i64>,
}

impl MediaFile {
    /// Creates a new [`MediaFile`]
    pub fn new(
        id: MediaFileId,
        media_id: MediaId,
        variant: MediaVariant,
        file_key: MediaFileKey,
        size_bytes: i64,
        mime_type: MimeType,
        duration_ms: Option<i64>,
    ) -> Self {
        Self {
            id,
            media_id,
            created_at: Utc::now(),
            variant,
            file_key,
            size_bytes,
            mime_type,
            duration_ms,
        }
    }

    /// Returns `true` if the current [`MediaFile`] has the specified duration
    pub fn has_duration(&self) -> bool {
        self.duration_ms.is_some()
    }
}
