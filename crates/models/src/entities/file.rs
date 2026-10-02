use chrono::{DateTime, Utc};
use mime::{MimeKind, MimeType};
use storage_types::StoragePath;

use crate::id::FileId;

/// Represents a stored file
///
/// Contains the file's storage location, size, MIME type
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

    /// File duration in milliseconds (for supported files)
    pub duration_ms: Option<i64>,
}

impl File {
    /// Creates a new [`File`]
    pub fn new(
        id: FileId,
        path: StoragePath,
        size_bytes: i64,
        mime_type: MimeType,
        duration_ms: Option<i64>,
    ) -> Self {
        Self {
            id,
            created_at: Utc::now(),
            path,
            size_bytes,
            mime_type,
            duration_ms,
        }
    }

    /// Returns the mime kind of this [`File`]
    pub fn mime_kind(&self) -> MimeKind {
        self.mime_type.kind()
    }
}
