use bytes::Bytes;
use tokio::io::AsyncRead;

use crate::{path::StoragePath, result::StorageError};

pub type DynWriter = dyn FileStorageWriter;
pub type BoxedWriter = Box<DynWriter>;

pub type DynReader = dyn AsyncRead + Send + Unpin;
pub type BoxedReader = Box<DynReader>;

/// File storage statistics
pub struct FsStats {
    pub available: u64,
    pub free: u64,
    pub total: u64,
}

/// Abstract file storage backend
#[async_trait::async_trait]
pub trait FileStorageBackend: Send + Sync {
    /// Creates a new [`FileStorageWriter`] at the specified [`StoragePath`] and returns it
    async fn write_stream(&self, path: &StoragePath) -> Result<BoxedWriter, StorageError>;

    /// Writes data to the file at the specified [`StoragePath`] and returns the number of bytes written
    async fn write(&self, path: &StoragePath, data: &mut DynReader) -> Result<usize, StorageError>;

    /// Opens a file [`BoxedReader`] at the specified [`StoragePath`] and returns it
    async fn read_stream(&self, path: &StoragePath) -> Result<BoxedReader, StorageError> {
        self.read_stream_seek(path, 0, None).await
    }

    /// Opens the file and reads its bytes from `start` to `end` (if specified),
    /// then returns a [`BoxedReader`] for the read range
    async fn read_stream_seek(
        &self,
        path: &StoragePath,
        start: u64,
        end: Option<u64>,
    ) -> Result<BoxedReader, StorageError>;

    /// Checks for the existence of a file at the specified [`StoragePath`]
    async fn exists(&self, path: &StoragePath) -> Result<bool, StorageError>;

    /// Changes the file path in the storage or returns false if the path is already taken
    async fn mv(&self, from: &StoragePath, to: &StoragePath) -> Result<bool, StorageError>;

    /// Reads the directory and returns the entries paths
    async fn read_dir(&self) -> Result<Vec<StoragePath>, StorageError>;

    /// Removes a file from the storage at the specified path
    /// or returns false if the path does not exist
    async fn remove(&self, path: &StoragePath) -> Result<bool, StorageError>;

    /// Returns file storage statistics
    async fn statfs(&self) -> Result<FsStats, StorageError>;
}

/// Transferring data from one storage to another
#[async_trait::async_trait]
pub trait FileStorageTransfer<T> {
    /// Moves a file from one storage to another
    async fn transfer(
        &self,
        from: &StoragePath,
        to: &StoragePath,
        to_storage: T,
    ) -> Result<bool, StorageError>;
}

/// Writer for the storage file
#[async_trait::async_trait]
pub trait FileStorageWriter {
    /// Writes the transmitted data to a file and returns the amount of bytes written
    async fn write(&mut self, data: Bytes) -> Result<usize, StorageError>;

    /// Completes writing to the file
    async fn flush(self: Box<Self>) -> Result<(), StorageError>;

    /// Interrupts writing to the file and deletes it
    async fn abort(self: Box<Self>) -> Result<(), StorageError>;
}
