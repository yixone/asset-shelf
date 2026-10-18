use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use mime::{MimeKind, MimeType};
use storage_types::StoragePath;

use crate::{
    id::FileGroupId,
    patches::FilePatch,
    types::{FileKey, FileVariant},
};

/// Represents a logical group of files belonging to the same media item
///
/// A file group owns its files. Each file belongs to exactly one file group
/// and is identified by its group and variant
///
/// A file group may contain multiple variants of the same media, such as
/// an original file, thumbnail, or preview
#[derive(Debug)]
pub struct FileGroup {
    /// File group identifier
    id: FileGroupId,

    /// Files included in the group
    files: HashMap<FileVariant, File>,
}

impl FileGroup {
    /// Creates a new [`FileGroup`]
    pub fn new(id: FileGroupId) -> Self {
        Self {
            id,
            files: HashMap::new(),
        }
    }

    /// Creates a new [`FileGroup`] containing a single [`File`] of the specified variant
    pub fn with_file(id: FileGroupId, variant: FileVariant, data: FileData) -> Self {
        let mut group = Self::new(id);
        group
            .files
            .insert(variant, File::new(group.id.clone(), variant, data));
        group
    }

    /// Returns a reference to the ID of this [`FileGroup`]
    pub fn id(&self) -> &FileGroupId {
        &self.id
    }

    #[inline]
    /// Returns the original media file
    pub fn original(&self) -> Option<&File> {
        self.file(FileVariant::Original)
    }

    /// Returns `true` if a file with the specified [`FileVariant`] is present
    /// in this [`FileGroup`]
    pub fn has_file(&self, variant: FileVariant) -> bool {
        self.files.contains_key(&variant)
    }

    /// Returns the file associated with the specified variant
    pub fn file(&self, variant: FileVariant) -> Option<&File> {
        self.files.get(&variant)
    }
}

#[cfg(feature = "dev")]
impl FileGroup {
    /// Creates [`FileGroup`] from persisted data
    ///
    /// This method is intended for use by persistence implementations
    /// when loading a file group from storage
    pub fn from_persistence(id: FileGroupId, files: impl IntoIterator<Item = File>) -> Self {
        let files = files.into_iter().map(|f| (f.key.variant(), f)).collect();
        FileGroup { id, files }
    }
}

/// Represents a file owned by [`FileGroup`]
///
/// Contains the file's storage location, size, MIME type
/// and optional duration for time-based media
#[derive(Debug)]
pub struct File {
    /// File key
    pub(crate) key: FileKey,

    /// File creation time
    pub(crate) created_at: DateTime<Utc>,

    /// File path in application storage
    pub(crate) path: StoragePath,

    /// File size in bytes
    pub(crate) size_bytes: u64,

    /// File MIME type
    pub(crate) mime_type: MimeType,

    /// File duration (for supported files)
    pub(crate) duration: Option<Duration>,
}

/// Data required to create a [`File`]
pub struct FileData {
    /// File path in application storage
    pub path: StoragePath,

    /// File size in bytes
    pub size_bytes: u64,

    /// File MIME type
    pub mime_type: MimeType,

    /// File duration (for supported files)
    pub duration: Option<Duration>,
}

impl File {
    /// Creates a new [`File`]
    pub fn new(group_id: FileGroupId, variant: FileVariant, data: FileData) -> Self {
        File {
            key: FileKey::new(group_id, variant),
            created_at: Utc::now(),
            path: data.path,
            size_bytes: data.size_bytes,
            mime_type: data.mime_type,
            duration: data.duration,
        }
    }

    /// Applies a patch to the [`File`]
    pub fn apply_patch(&mut self, patch: FilePatch) {
        patch.apply_domain(self);
    }

    /// Returns a reference to the key of this [`File`]
    pub fn key(&self) -> &FileKey {
        &self.key
    }

    #[inline]
    /// Returns the ID of the group to which this [`File`] belongs
    pub fn group_id(&self) -> &FileGroupId {
        self.key().group_id()
    }

    #[inline]
    /// Returns a variant of this [`File`]
    pub fn variant(&self) -> FileVariant {
        self.key().variant()
    }

    /// Returns this [`File`] creation time
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns a reference to this [`File`]'s path in application storage
    pub fn path(&self) -> &StoragePath {
        &self.path
    }

    /// Returns this [`File`] size in bytes
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    /// Returns the MIME type of this [`File`]
    pub fn mime_type(&self) -> MimeType {
        self.mime_type
    }

    #[inline]
    /// Returns the MIME kind of this [`File`]
    pub fn mime_kind(&self) -> MimeKind {
        self.mime_type().kind()
    }

    /// Returns the duration of this [`File`] (if supported)
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }
}

#[cfg(feature = "dev")]
impl File {
    /// Creates [`File`] from persisted data
    ///
    /// This method is intended for use by persistence implementations
    /// when loading a file from storage
    pub fn from_persistence(
        key: (FileGroupId, FileVariant),
        created_at: DateTime<Utc>,
        data: FileData,
    ) -> Self {
        File {
            key: FileKey::new(key.0, key.1),
            created_at,
            path: data.path,
            size_bytes: data.size_bytes,
            mime_type: data.mime_type,
            duration: data.duration,
        }
    }
}
