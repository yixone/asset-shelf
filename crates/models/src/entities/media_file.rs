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

    pub duration_milis: Option<i64>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum MediaVariant {
    #[default]
    Original,
    Thumbnail,
    LoopPreview,
}

impl MediaVariant {
    pub const fn as_str(&self) -> &'static str {
        match self {
            MediaVariant::Original => "original",
            MediaVariant::Thumbnail => "thumbnail",
            MediaVariant::LoopPreview => "loop_preview",
        }
    }
}

impl std::fmt::Display for MediaVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
