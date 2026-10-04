use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mime::{MimeKind, MimeType};
use storage_types::StoragePath;

use crate::{
    EntityError,
    id::FileGroupId,
    patches::FilePatch,
    ports::{FileDatabase, FileGroupDatabase, FileGroupWriter, FileWriter},
    result::Result,
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

    /// Original file name provided with the media
    original_file_name: Option<String>,
}

impl FileGroup {
    /// Creates and persists a new [`FileGroup`]
    ///
    /// The group is created without any files
    pub async fn create<DB>(
        id: FileGroupId,
        original_file_name: Option<String>,
        db: &mut DB,
    ) -> Result<Self>
    where
        DB: FileGroupWriter,
    {
        let group = FileGroup {
            id,
            files: HashMap::new(),
            original_file_name,
        };
        db.insert_file_group(&group).await?;
        Ok(group)
    }

    /// Creates and persists a [`FileGroup`] with an initial [`File`]
    ///
    /// The file group and its file are persisted as part of the same operation.
    /// The returned group contains the persisted file
    pub async fn create_with_file<DB>(
        id: FileGroupId,
        original_file_name: Option<String>,
        data: FileData,
        db: &mut DB,
    ) -> Result<Self>
    where
        DB: FileGroupWriter + FileWriter,
    {
        let mut group = FileGroup::create(id, original_file_name, db).await?;

        let file = File {
            key: FileKey::new(group.id.clone(), data.variant),
            created_at: Utc::now(),
            path: data.path,
            size_bytes: data.size_bytes,
            mime_type: data.mime_type,
            duration_ms: data.duration_ms,
        };

        db.insert_file(&file).await?;
        group.files.insert(data.variant, file);

        Ok(group)
    }

    /// Loads a [`FileGroup`] by its identifier
    ///
    /// Returns [`EntityError::NotFound`] if the file group does not exist
    pub async fn load<DB>(id: &FileGroupId, db: &DB) -> Result<Self>
    where
        DB: FileGroupDatabase,
    {
        db.get_file_group(id).await?.ok_or(EntityError::NotFound)
    }

    /// Loads a [`File`] with the specified variant from a [`FileGroup`]
    ///
    /// Returns [`EntityError::NotFound`] if the file group
    /// or the specified variant does not exist
    pub async fn load_file<DB>(
        id: impl AsRef<FileGroupId>,
        variant: FileVariant,
        db: &DB,
    ) -> Result<File>
    where
        DB: FileDatabase,
    {
        db.get_file(&FileKey::new(id.as_ref().clone(), variant))
            .await?
            .ok_or(EntityError::NotFound)
    }

    /// Creates and persists a [`File`] belonging to this [`FileGroup`]
    ///
    /// Returns [`EntityError::AlreadyExists`] if the specified variant is
    /// already present in the group
    pub async fn create_file<DB>(&mut self, data: FileData, db: &mut DB) -> Result<()>
    where
        DB: FileWriter,
    {
        if self.has_file(data.variant) {
            return Err(EntityError::AlreadyExists);
        }

        let file = File {
            key: FileKey::new(self.id.clone(), data.variant),
            created_at: Utc::now(),
            path: data.path,
            size_bytes: data.size_bytes,
            mime_type: data.mime_type,
            duration_ms: data.duration_ms,
        };

        db.insert_file(&file).await?;
        self.files.insert(data.variant, file);

        Ok(())
    }

    /// Deletes the [`FileGroup`] and its attached files from persistence
    ///
    /// Consumes the group because it can no longer represent a
    /// persisted entity after deletion
    pub async fn delete<DB>(self, db: &DB) -> Result<()>
    where
        DB: FileGroupDatabase,
    {
        db.delete_file_group(&self.id).await?;
        Ok(())
    }

    /// Deletes the [`File`] with the specified variant from this [`FileGroup`]
    ///
    /// Returns [`EntityError::NotFound`] if the specified file does not exist
    pub async fn delete_file<DB>(&mut self, variant: FileVariant, db: &DB) -> Result<()>
    where
        DB: FileDatabase,
    {
        if !db
            .delete_file(&FileKey::new(self.id.clone(), variant))
            .await?
        {
            return Err(EntityError::NotFound);
        }

        self.files.remove(&variant);

        Ok(())
    }

    /// Updates the [`File`] with the specified variant using the given patch
    ///
    /// Returns [`EntityError::NotFound`] if the specified file does not exist
    ///
    /// The domain model is updated only after the persistence operation succeeds
    pub async fn update_file<DB>(
        &mut self,
        variant: FileVariant,
        patch: FilePatch,
        db: &DB,
    ) -> Result<()>
    where
        DB: FileDatabase,
    {
        let file = self.file_mut(variant).ok_or(EntityError::NotFound)?;

        if !db.update_file(file.key(), &patch).await? {
            return Err(EntityError::NotFound);
        }

        patch.apply_domain(file);

        Ok(())
    }

    /// Returns the file associated with the specified variant
    pub fn file(&self, variant: FileVariant) -> Option<&File> {
        self.files.get(&variant)
    }

    /// Returns the file associated with the specified variant as mutable reference
    fn file_mut(&mut self, variant: FileVariant) -> Option<&mut File> {
        self.files.get_mut(&variant)
    }

    /// Returns the files iter of this [`FileGroup`]
    pub fn files_iter(&self) -> impl Iterator<Item = &File> {
        self.files.values()
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

    /// Returns a reference to the id of this [`FileGroup`]
    pub fn id(&self) -> &FileGroupId {
        &self.id
    }

    /// Returns the original file name of this [`FileGroup`]
    pub fn original_file_name(&self) -> Option<&str> {
        self.original_file_name.as_deref()
    }
}

#[cfg(feature = "dev")]
impl FileGroup {
    /// Creates [`FileGroup`] from persisted data
    ///
    /// This method is intended for use by persistence implementations
    /// when loading a file group from storage
    pub fn from_persistence(
        id: FileGroupId,
        files: impl IntoIterator<Item = File>,
        original_file_name: Option<String>,
    ) -> Self {
        let files = files.into_iter().map(|f| (f.key.variant(), f)).collect();
        FileGroup {
            id,
            files,
            original_file_name,
        }
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

    /// File duration in milliseconds (for supported files)
    pub(crate) duration_ms: Option<i64>,
}

impl File {
    /// Returns a reference to the key of this [`File`]
    pub fn key(&self) -> &FileKey {
        &self.key
    }

    /// Returns the [`File`] creation time
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns a reference to the [`File`] path in
    /// application storage
    pub fn path(&self) -> &StoragePath {
        &self.path
    }

    /// Returns the [`File`] size in bytes
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    /// Returns the mime type of this [`File`]
    pub fn mime_type(&self) -> MimeType {
        self.mime_type
    }

    /// Returns the mime kind of this [`File`]
    pub fn mime_kind(&self) -> MimeKind {
        self.mime_type.kind()
    }

    /// Returns the [`File`] duration in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.duration_ms
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
            duration_ms: data.duration_ms,
        }
    }
}

/// Data required to create or reconstruct a [`File`]
pub struct FileData {
    /// File variant within its [`FileGroup`]
    pub variant: FileVariant,

    /// File path in application storage
    pub path: StoragePath,

    /// File size in bytes
    pub size_bytes: u64,

    /// File MIME type
    pub mime_type: MimeType,

    /// File duration in milliseconds (for supported files)
    pub duration_ms: Option<i64>,
}
