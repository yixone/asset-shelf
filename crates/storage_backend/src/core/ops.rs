use tokio::io::AsyncRead;

use crate::core::{path::StoragePath, result::Result};

#[async_trait::async_trait]
pub trait StorageBackend {
    async fn open_writer(&self, path: &StoragePath) -> Result<Box<dyn StorageWriter>>;
    async fn get_reader(&self, path: &StoragePath) -> Result<Box<dyn AsyncRead + Send + Unpin>>;

    async fn rename(&self, from: &StoragePath, to: &StoragePath) -> Result<bool>;

    async fn remove(&self, path: &StoragePath) -> Result<bool>;
}

#[async_trait::async_trait]
pub trait StorageWriter {
    async fn write_chunk(&mut self, data: bytes::Bytes) -> Result<()>;

    async fn finalize(self: Box<Self>) -> Result<()>;
    async fn abort(self: Box<Self>) -> Result<()>;
}
