use chrono::{DateTime, Utc};
use mime::{MimeKind, MimeType};
use storage_types::StoragePath;

use crate::id::FileId;

/// Represents a stored file
///
/// Contains the file's storage location, variant, MIME type, size
/// and optional duration for time-based media
#[derive(Debug)]
pub struct File {
    /// File identifier
    pub id: FileId,

    /// File creation time
    pub created_at: DateTime<Utc>,

    /// File path in application storage
    pub path: StoragePath,

    /// File size in bytes
    pub size_bytes: i64,

    /// File MIME type
    pub mime_type: MimeType,
    /// File MIME kind
    pub mime_kind: MimeKind,

    /// File duration in milliseconds (for supported files)
    pub duration_ms: Option<i64>,
}
