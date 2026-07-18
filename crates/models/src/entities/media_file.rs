use chrono::{DateTime, Utc};
use mimetype::MimeType;

use crate::types::{MediaFileId, MediaId};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct MediaFile {
    pub id: MediaFileId,
    pub media_id: MediaId,

    pub variant: MediaVariant,
    pub storage_key: String,

    pub created_at: DateTime<Utc>,

    pub size_bytes: i64,
    pub mimetype: MimeType,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MediaVariant {
    Original,
    Thumbnail,
}
