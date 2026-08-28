use std::{
    io::ErrorKind,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use result::{Error, Result, create_error, error::ResultExt};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter},
};

use crate::{
    StoragePath,
    backend::{BoxedReader, BoxedWriter, FileWriter, StorageBackend},
};

/// File storage backend for a native file system
#[derive(Debug)]
pub struct NativeFsStorageBackend {
    /// Storage root directory
    root: PathBuf,
}

/// Writer for a file in the native file system
pub struct NativeFsStorageWriter {
    /// File Writer
    writer: BufWriter<File>,
    /// Path to the file being written to
    path: PathBuf,
}

impl NativeFsStorageBackend {
    /// Creates new [`NativeFsStorageBackend`] with the specified root
    pub async fn new<P>(root: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let root = root.as_ref();
        tokio::fs::create_dir_all(root).await.to_app_err()?;
        Ok(NativeFsStorageBackend {
            root: root.to_path_buf(),
        })
    }

    /// Creates all parent directories for the specified path.
    async fn create_parents(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.to_app_err()?;
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

    /// Creates an absolute path to the file in the storage
    pub fn resolve_path(&self, path: &StoragePath) -> PathBuf {
        self.root.join(path.to_path())
    }
}

#[async_trait::async_trait]
impl StorageBackend for NativeFsStorageBackend {
    async fn create(&self, path: &StoragePath) -> Result<BoxedWriter> {
        let path = self.resolve_path(path);
        self.create_parents(&path).await?;

        let file = File::create_new(&path).await.to_app_err()?;
        let writer = BufWriter::with_capacity(32 * 1024, file);

        let writer = NativeFsStorageWriter { writer, path };
        Ok(Box::new(writer))
    }

    async fn move_from_local(&self, from: &Path, dest: &StoragePath) -> Result<usize> {
        let dest = self.resolve_path(dest);
        self.create_parents(&dest).await?;

        tokio::fs::rename(from, &dest).await.to_app_err()?;

        let meta = tokio::fs::metadata(dest).await.to_app_err()?;
        let size = meta.size() as usize;

        Ok(size)
    }

    async fn read(&self, path: &StoragePath) -> Result<BoxedReader> {
        let path = self.resolve_path(path);
        match File::open(path).await {
            Ok(f) => Ok(Box::new(f)),
            Err(e) if e.kind() == ErrorKind::NotFound => Err(create_error!(NotFound)),
            Err(e) => Err(Error::internal(e)),
        }
    }

    async fn read_ranged(
        &self,
        path: &StoragePath,
        start: u64,
        end: Option<u64>,
    ) -> Result<BoxedReader> {
        let path = self.resolve_path(path);
        let mut file = match File::open(path).await {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Err(create_error!(NotFound)),
            Err(e) => return Err(Error::internal(e)),
        };

        file.seek(std::io::SeekFrom::Start(start))
            .await
            .to_app_err()?;

        match end {
            Some(end) => {
                let to_read = end.saturating_sub(start) + 1;
                Ok(Box::new(file.take(to_read)))
            }
            None => Ok(Box::new(file)),
        }
    }

    async fn exists(&self, path: &StoragePath) -> Result<bool> {
        let path = self.resolve_path(path);
        tokio::fs::try_exists(path).await.to_app_err()
    }

    async fn mv(&self, from: &StoragePath, to: &StoragePath) -> Result<bool> {
        let from = self.resolve_path(from);
        let to = self.resolve_path(to);

        // TODO: Fix TOCTOU
        if tokio::fs::try_exists(&to).await.to_app_err()? {
            return Ok(false);
        }

        self.create_parents(&to).await?;
        tokio::fs::rename(&from, &to).await.to_app_err()?;

        Ok(true)
    }

    async fn remove(&self, path: &StoragePath) -> Result<bool> {
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
            Err(e) => Err(create_error!(source = e)),
        }
    }
}

#[async_trait::async_trait]
impl FileWriter for NativeFsStorageWriter {
    async fn write(&mut self, data: bytes::Bytes) -> Result<usize> {
        self.writer.write_all(&data).await.to_app_err()?;
        Ok(data.len())
    }

    async fn flush(mut self: Box<Self>) -> Result<()> {
        self.writer.flush().await.to_app_err()?;
        Ok(())
    }

    async fn abort(mut self: Box<Self>) -> Result<()> {
        drop(self.writer);
        tokio::fs::remove_file(&self.path).await.to_app_err()?;
        Ok(())
    }
}
