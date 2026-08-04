use std::path::{Path, PathBuf};

use result::Result;

use crate::{Storage, StoragePath};

/// A reserved path where a file can be written;
/// upon publication, it will be saved to the storage.
pub struct ReservedFile<'a> {
    /// The storage that reserved the file
    pub(crate) owner: &'a Storage,
    /// The path where the reserved file will be published in the storage
    pub(crate) publish_path: StoragePath,

    /// Path to the reserved file in the local file system
    pub(crate) path: PathBuf,

    /// If `false` at the time the destructor is called, the reserved file will be deleted.
    pub(crate) need_drop: bool,
}

/// Result of publishing the reserved file
pub struct FilePublishingResult {
    /// The path in the storage where the file would be published
    pub path: StoragePath,
    /// Size of the published file in bytes
    pub size_bytes: usize,
}

impl<'a> ReservedFile<'a> {
    /// Publishes the [`ReservedFile`], deletes it from the local disk,
    /// and returns [`FilePublishingResult`]
    pub async fn publish(mut self) -> Result<FilePublishingResult> {
        let publish_path = self.publish_path.clone();

        // TODO!

        // Disables the automatic file deletion guard
        self.need_drop = false;

        // Returns the result of the file publication
        Ok(FilePublishingResult {
            path: publish_path,
            size_bytes: 0,
        })
    }

    /// Triggers the deletion of the reserved file
    pub async fn abort(mut self) {
        // If the file was not used, it disables the guard and exits the function
        if let Ok(exists) = tokio::fs::try_exists(&self.path).await
            && !exists
        {
            self.need_drop = false;
            return;
        }

        // Deletes a file from the disk
        if let Err(e) = tokio::fs::remove_file(&self.path).await {
            tracing::error!(err = ?e, "Failed to delete reserved file on abort");
        }
    }

    /// Returns a local path for this [`ReservedFile`]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ReservedFile<'_> {
    fn drop(&mut self) {
        // Checking that the file was published
        if !self.need_drop {
            return;
        }

        // Checking the situation where a reserved file was not used
        if !self.path.exists() {
            return;
        }

        // Automatic deletion of an unpublished reserved file upon destructor invocation
        if let Err(e) = std::fs::remove_file(&self.path) {
            tracing::error!(err = ?e, "Failed to delete unpublished reserved file");
        }
    }
}

/// A temporary file that was written to the temporary directory of the local storage
pub struct LocalFile {
    /// Temporary path where the local file was saved
    pub(crate) path: PathBuf,

    /// If `true`, the local file will be automatically cleaned up when the destructor is called
    pub(crate) need_drop: bool,
}

impl LocalFile {
    /// Returns a reference to the path of this [`LocalFile`]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LocalFile {
    fn drop(&mut self) {
        if !self.need_drop {
            return;
        }

        // Automatic deletion of a temp local file upon destructor invocation
        if let Err(e) = std::fs::remove_file(&self.path) {
            tracing::error!(err = ?e, "Failed to delete temp local file");
        }
    }
}
