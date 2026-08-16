use chrono::{DateTime, Utc};
use mimetype::MimeType;

use crate::{
    media::MediaVariant,
    types::{MediaFileId, MediaId},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct MediaFile {
    pub id: MediaFileId,
    pub media_id: MediaId,

    pub variant: MediaVariant,
    pub storage_path: String,

    pub created_at: DateTime<Utc>,

    pub size_bytes: i64,
    pub mimetype: MimeType,

    pub duration_ms: Option<i64>,
}

impl MediaFile {
    /// Creates a new [`MediaFile`]
    pub fn new(
        id: MediaFileId,
        media_id: MediaId,
        variant: MediaVariant,
        storage_path: String,
        size_bytes: i64,
        mimetype: MimeType,
        duration_ms: Option<i64>,
    ) -> Self {
        Self {
            id,
            media_id,
            variant,
            storage_path,
            created_at: Utc::now(),
            size_bytes,
            mimetype,
            duration_ms,
        }
    }
}
