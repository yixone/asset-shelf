use chrono::{DateTime, Utc};
use mimetype::MimeType;

use crate::types::{MediaFileId, MediaId};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct MediaFile {
    pub id: MediaFileId,
    pub media_id: MediaId,

    pub variant: MediaVariant,
    pub storage_path: String,

    pub created_at: DateTime<Utc>,

    pub size_bytes: i64,
    pub mimetype: MimeType,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MediaVariant {
    Original,
    Thumbnail,
}

impl std::fmt::Display for MediaVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MediaVariant::Original => "original",
            MediaVariant::Thumbnail => "thumbnail",
        })
    }
}
