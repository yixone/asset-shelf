use std::path::{Path, PathBuf};

use result::{Result, create_error, error::ResultExt};

use crate::{Storage, StoragePath};

/// A reserved path where a file can be written;
/// upon publication, it will be saved to the storage.
pub struct ReservedFile<'a> {
    /// The [`Storage`] that reserved the file
    pub(crate) owner: &'a Storage,

    /// The path where the reserved file will be published in the storage
    pub(crate) publish_path: StoragePath,

    /// The temporary path where the reserved
    /// file is located in the temporary section of the storage
    pub(crate) temp_path: StoragePath,

    /// Path to the reserved file in the local file system
    pub(crate) path: PathBuf,

    /// If `true` at the time the destructor is called, the reserved file will be deleted.
    pub(crate) need_cleanup: bool,
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
        // Handles the situation where a reserved file was not used but an attempt is made to publish it
        let exists = tokio::fs::try_exists(&self.path).await.to_app_err()?;
        if !exists {
            return Err(create_error!(NotFound));
        }

        // Gets the paths
        let publish_path = self.publish_path.clone();
        let temp_path = &self.temp_path;

        // Moves the reserved file from the temporary section to the global section
        let size_bytes = self
            .owner
            .move_to_global_section(temp_path, &publish_path)
            .await?;

        // Disables the automatic file deletion guard
        self.need_cleanup = false;

        // Returns the result of the file publication
        Ok(FilePublishingResult {
            path: publish_path,
            size_bytes,
        })
    }

    /// Triggers the deletion of the reserved file
    pub async fn abort(mut self) {
        // If the file was not used, it disables the guard and exits the function
        if let Ok(exists) = tokio::fs::try_exists(&self.path).await
            && !exists
        {
            self.need_cleanup = false;
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
        if !self.need_cleanup {
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
    pub(crate) need_cleanup: bool,
}

impl LocalFile {
    /// Returns a reference to the path of this [`LocalFile`]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LocalFile {
    fn drop(&mut self) {
        if !self.need_cleanup {
            return;
        }

        // Automatic deletion of a temp local file upon destructor invocation
        if let Err(e) = std::fs::remove_file(&self.path) {
            tracing::error!(err = ?e, "Failed to delete temp local file");
        }
    }
}

/// Unconfirmed uploaded file located in the temporary section of the [`Storage`]
pub struct UncommitedFile<'a> {
    /// The [`Storage`] that uploads this file
    pub(crate) owner: &'a Storage,

    /// Size of the uncommitted file in bytes
    pub size_bytes: usize,

    /// The temporary path where an uncommitted file resides
    /// in the temporary section of the repository
    pub(crate) temp_path: StoragePath,

    /// The global path at which the file will be committed
    /// to the permanent section of the storage
    pub(crate) global_path: StoragePath,

    /// If `true` at the time the destructor is called, the uncommited file will be deleted
    pub(crate) need_cleanup: bool,
}

pub struct CommitedFile {
    /// Size of the file in bytes
    pub size_bytes: usize,

    /// The path under which the file was committed
    pub global_path: StoragePath,
}

impl<'a> UncommitedFile<'a> {
    /// Commits the temporary file, saving it to the permanent section of the storage
    pub async fn commit(mut self) -> Result<CommitedFile> {
        // Moving a temporary file to a global section
        self.owner
            .move_to_global_section(&self.temp_path, &self.global_path)
            .await?;

        // Notes that the temporary file does not need to be
        // deleted after being moved to the global section
        self.need_cleanup = false;

        // Creates and returns a commited file data
        let file = CommitedFile {
            size_bytes: self.size_bytes,
            global_path: self.global_path.clone(),
        };
        Ok(file)
    }

    /// Returns a reference to the global path of this [`UncommitedFile`]
    pub fn global_path(&self) -> &StoragePath {
        &self.global_path
    }
}

impl Drop for UncommitedFile<'_> {
    fn drop(&mut self) {
        if !self.need_cleanup {
            return;
        }

        // Automatic deletion of a uncommited file upon destructor invocation
        let real_path = self.owner.temp.resolve_path(&self.temp_path);
        if let Err(e) = std::fs::remove_file(real_path) {
            tracing::error!(err = ?e, "Failed to delete temp local file");
        }
    }
}
