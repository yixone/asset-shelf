use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mime::{MimeKind, MimeType};
use storage_types::StoragePath;

use crate::{
    id::{FileGroupId, FileId},
    types::FileVariant,
};

#[derive(Debug)]
pub struct FileGroup {
    /// File group identifier
    pub id: FileGroupId,

    /// Files included in the group
    pub files: HashMap<FileVariant, File>,
}

impl FileGroup {
    /// Returns the file associated with the specified variant
    pub fn file(&self, variant: FileVariant) -> Option<&File> {
        self.files.get(&variant)
    }

    /// Returns `true` if the specified file variant is present
    pub fn has_file(&self, variant: FileVariant) -> bool {
        self.files.contains_key(&variant)
    }

    /// Returns the original media file
    pub fn original(&self) -> Option<&File> {
        self.files.get(&FileVariant::Original)
    }

    /// Returns the thumbnail media file
    pub fn thumbnail(&self) -> Option<&File> {
        self.files.get(&FileVariant::Thumbnail)
    }
}

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
