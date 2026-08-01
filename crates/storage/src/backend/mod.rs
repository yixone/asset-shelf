//! Internal implementations of the file storage

use bytes::Bytes;
use result::Result;
use tokio::io::AsyncRead;

use crate::backend::path::StoragePath;

pub mod fs;
pub mod s3;

pub mod path;

pub type BoxedWriter = Box<dyn FileWriter>;
pub type BoxedReader = Box<dyn AsyncRead + Send + Unpin>;

/// Abstract file storage backend
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Creates a new [`FileWriter`] at the specified [`StoragePath`] and returns it
    async fn create(&self, path: &StoragePath) -> Result<BoxedWriter>;

    /// Opens a file reader at the specified [`StoragePath`] and returns it
    async fn read(&self, path: &StoragePath) -> Result<BoxedReader>;

    /// Checks for the existence of a file at the specified [`StoragePath`]
    async fn exists(&self, path: &StoragePath) -> Result<bool>;

    /// Changes the file path in the storage or returns false if the path is already taken
    async fn mv(&self, from: &StoragePath, to: &StoragePath) -> Result<bool>;

    /// Removes a file from the storage at the specified path
    /// or returns false if the path does not exist
    async fn remove(&self, path: &StoragePath) -> Result<bool>;
}

/// Writer for the storage file
#[async_trait::async_trait]
pub trait FileWriter: Send + Sync {
    /// Writes the transmitted data to a file and returns the amount of bytes written
    async fn write(&mut self, data: Bytes) -> Result<usize>;

    /// Completes writing to the file
    async fn flush(self: Box<Self>) -> Result<()>;

    /// Interrupts writing to the file and deletes it
    async fn abort(self: Box<Self>) -> Result<()>;
}
