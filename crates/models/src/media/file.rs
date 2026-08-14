use chrono::{DateTime, Utc};
use mimetype::MimeType;

use crate::{
    media::MediaVariant,
    types::{MediaFileId, MediaId},
};

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

    pub duration_ms: Option<i64>,
}
