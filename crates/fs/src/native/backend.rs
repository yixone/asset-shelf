//! Implementation of a file storage backend for a local file system

use std::path::{Path, PathBuf};

use crate::{
    backend::{BoxedReader, BoxedWriter, DynReader, FileStorageBackend, FsStats},
    path::StoragePath,
    result::StorageError,
};

/// File storage backend for the local file system
pub struct FsStorageBackend {
    root: PathBuf,
    read_only: bool,
}

impl FsStorageBackend {
    /// Opens the file storage in the specified directory
    pub fn open(
        root: impl Into<PathBuf>,
        read_only: bool,
    ) -> Result<FsStorageBackend, StorageError> {
        let root = root.into();

        // Creates a file storage directory
        std::fs::create_dir_all(&root)?;

        // Creates a file storage entity
        let fs = FsStorageBackend { root, read_only };

        Ok(fs)
    }

    /// Returns an error if the file storage was created in read-only mode
    pub fn resolve_write(&self) -> Result<(), StorageError> {
        if self.read_only {
            Err(StorageError::ReadOnly)
        } else {
            Ok(())
        }
    }

    /// Creates an absolute path to the file in the storage
    pub fn resolve_path(&self, path: impl AsRef<StoragePath>) -> PathBuf {
        let path = path.as_ref();
        self.root.join(path.as_path())
    }

    /// Creates all parent directories for the specified path.
    async fn create_parents(&self, path: &Path) -> Result<(), StorageError> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    /// Deletes all empty parent directories for the specified path.
    /// Stops deletion if a directory contains files
    async fn delete_parents_safely(&self, path: &Path) {
        let mut p = path;
        while let Some(parent) = p.parent() {
            if parent == self.root {
                break;
            }
            p = parent;
            match tokio::fs::remove_dir(parent).await {
                Ok(_) => (),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
                Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
                Err(e) => {
                    tracing::error!(
                        err = ?e,
                        "Failed to delete directory parent"
                    );
                    return;
                }
            }
        }
        tracing::info!(
            from = ?path, to = ?p,
            "Parent directories have been removed"
        );
    }
}

#[async_trait::async_trait]
impl FileStorageBackend for FsStorageBackend {
    /// Creates a new [`FileStorageWriter`] at the specified [`StoragePath`] and returns it
    async fn write_stream(&self, path: &StoragePath) -> Result<BoxedWriter, StorageError> {
        todo!()
    }

    /// Writes data to the file at the specified [`StoragePath`] and returns the number of bytes written
    async fn write(&self, path: &StoragePath, data: &mut DynReader) -> Result<usize, StorageError> {
        todo!()
    }

    /// Opens the file and reads its bytes from `start` to `end` (if specified),
    /// then returns a [`BoxedReader`] for the read range
    async fn read_stream_seek(
        &self,
        path: &StoragePath,
        start: u64,
        end: Option<u64>,
    ) -> Result<BoxedReader, StorageError> {
        todo!()
    }

    /// Checks for the existence of a file at the specified [`StoragePath`]
    async fn exists(&self, path: &StoragePath) -> Result<bool, StorageError> {
        todo!()
    }

    /// Changes the file path in the storage or returns false if the path is already taken
    async fn mv(&self, from: &StoragePath, to: &StoragePath) -> Result<bool, StorageError> {
        todo!()
    }

    /// Reads the directory and returns the entries paths
    async fn read_dir(&self) -> Result<Vec<StoragePath>, StorageError> {
        todo!()
    }

    /// Removes a file from the storage at the specified path
    /// or returns false if the path does not exist
    async fn remove(&self, path: &StoragePath) -> Result<bool, StorageError> {
        todo!()
    }

    /// Returns file storage statistics
    async fn statfs(&self) -> Result<FsStats, StorageError> {
        todo!()
    }
}
