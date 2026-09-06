//! Implementation of a file storage backend for a local file system

use std::path::{Path, PathBuf};

use bytes::Bytes;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter},
};

use crate::{
    backend::{
        BoxedReader, BoxedWriter, DynReader, FileStorageBackend, FileStorageTransfer,
        FileStorageWriter, FsStats,
    },
    helpers::future::asyncify,
    native,
    path::StoragePath,
    result::StorageError,
};

/// File storage backend for the local file system
pub struct FsStorageBackend {
    /// Storage root directory
    root: PathBuf,
    /// If `true`, the file storage is in `read-only` mode
    read_only: bool,
}

/// Writer for a file in the local file system
pub struct FsStorageWriter {
    /// File Writer
    writer: BufWriter<File>,
    /// Path to the file being written to
    path: PathBuf,
}

impl FsStorageBackend {
    /// Opens the file storage in the specified directory
    pub fn open(
        root: impl Into<PathBuf>,
        read_only: bool,
    ) -> Result<FsStorageBackend, StorageError> {
        let root = root.into();

        // Creates a file storage directory
        if !root.exists() {
            if read_only {
                return Err(StorageError::ReadOnly);
            }

            std::fs::create_dir_all(&root)?;
        }

        // Creates a file storage entity
        let fs = FsStorageBackend { root, read_only };

        Ok(fs)
    }

    /// Returns an error if the file storage was created in read-only mode
    fn resolve_write(&self) -> Result<(), StorageError> {
        if self.read_only {
            Err(StorageError::ReadOnly)
        } else {
            Ok(())
        }
    }

    /// Creates an absolute path to the file in the storage
    fn resolve_path(&self, path: &StoragePath) -> PathBuf {
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
        self.resolve_write()?;

        let path = self.resolve_path(path);
        self.create_parents(&path).await?;

        let file = File::create(&path).await?;
        let writer = BufWriter::with_capacity(32 * 1024, file);

        let writer = FsStorageWriter { writer, path };
        Ok(Box::new(writer))
    }

    /// Writes data to the file at the specified [`StoragePath`] and returns the number of bytes written
    async fn write(&self, path: &StoragePath, data: &mut DynReader) -> Result<usize, StorageError> {
        self.resolve_write()?;

        let mut file = File::create(self.resolve_path(path)).await?;

        let b_written = tokio::io::copy(data, &mut file).await?;

        Ok(b_written as usize)
    }

    /// Opens the file and reads its bytes from `start` to `end` (if specified),
    /// then returns a [`BoxedReader`] for the read range
    async fn read_stream_seek(
        &self,
        path: &StoragePath,
        start: u64,
        end: Option<u64>,
    ) -> Result<BoxedReader, StorageError> {
        let path = self.resolve_path(path);

        let mut file = match File::open(path).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound);
            }
            Err(e) => return Err(StorageError::Io(e)),
        };

        file.seek(std::io::SeekFrom::Start(start)).await?;

        match end {
            Some(end) => {
                let to_read = end.saturating_sub(start) + 1;
                Ok(Box::new(file.take(to_read)))
            }
            None => Ok(Box::new(file)),
        }
    }

    /// Checks for the existence of a file at the specified [`StoragePath`]
    async fn exists(&self, path: &StoragePath) -> Result<bool, StorageError> {
        let exists = tokio::fs::try_exists(self.resolve_path(path)).await?;
        Ok(exists)
    }

    /// Changes the file path in the storage or returns false if the path is already taken
    async fn mv(&self, from: &StoragePath, to: &StoragePath) -> Result<bool, StorageError> {
        self.resolve_write()?;

        let from = self.resolve_path(from);
        let to = self.resolve_path(to);

        // FIXME: Fix TOCTOU via `renameat2`
        if tokio::fs::try_exists(&to).await? {
            return Ok(false);
        }

        self.create_parents(&to).await?;
        tokio::fs::rename(&from, &to).await?;

        Ok(true)
    }

    /// Reads the directory and returns the entries paths
    async fn read_dir(&self) -> Result<Vec<StoragePath>, StorageError> {
        todo!()
    }

    /// Removes a file from the storage at the specified path
    /// or returns false if the path does not exist
    async fn remove(&self, path: &StoragePath) -> Result<bool, StorageError> {
        self.resolve_write()?;

        let path = self.resolve_path(path);

        match tokio::fs::remove_file(&path).await {
            Ok(_) => {
                tracing::info!(
                    path = ?path,
                    "File removed"
                );
                self.delete_parents_safely(&path).await;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(StorageError::Io(e)),
        }
    }

    /// Returns file storage statistics
    async fn statfs(&self) -> Result<FsStats, StorageError> {
        let root = self.root.clone();
        asyncify(move || native::stats::statvfs(&root)).await?
    }
}

#[async_trait::async_trait]
impl FileStorageWriter for FsStorageWriter {
    /// Writes the transmitted data to a file and returns the amount of bytes written
    async fn write(&mut self, data: Bytes) -> Result<usize, StorageError> {
        self.writer.write_all(&data).await?;
        Ok(data.len())
    }

    /// Completes writing to the file
    async fn flush(mut self: Box<Self>) -> Result<(), StorageError> {
        self.writer.flush().await?;
        Ok(())
    }

    /// Interrupts writing to the file and deletes it
    async fn abort(mut self: Box<Self>) -> Result<(), StorageError> {
        drop(self.writer);
        tokio::fs::remove_file(&self.path).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl FileStorageTransfer<FsStorageBackend> for FsStorageBackend {
    /// Moves a file from one storage to another
    async fn transfer(
        &self,
        from: &StoragePath,
        to: &StoragePath,
        target: &FsStorageBackend,
    ) -> Result<bool, StorageError> {
        let f_path = self.resolve_path(from);
        let t_path = target.resolve_path(to);

        // FIXME: Fix TOCTOU via `renameat2`
        if tokio::fs::try_exists(&t_path).await? {
            return Ok(false);
        }

        target.create_parents(&t_path).await?;
        tokio::fs::rename(&f_path, &t_path).await?;

        Ok(true)
    }

    /// Copies a file from one storage to another
    async fn copy(
        &self,
        from: &StoragePath,
        to: &StoragePath,
        target: &FsStorageBackend,
    ) -> Result<bool, StorageError> {
        let f_path = self.resolve_path(from);
        let t_path = target.resolve_path(to);

        // FIXME: Fix TOCTOU via `renameat2`
        if tokio::fs::try_exists(&t_path).await? {
            return Ok(false);
        }

        target.create_parents(&t_path).await?;
        tokio::fs::copy(&f_path, &t_path).await?;

        Ok(true)
    }
}
