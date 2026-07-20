use std::path::{Path, PathBuf};

use tokio::{
    fs::File,
    io::{AsyncRead, AsyncWriteExt, BufWriter},
};

use crate::core::{
    ops::{AbstractStorageBackend, AbstractStorageWriter},
    path::StoragePath,
    result::Result,
};

#[derive(Debug)]
pub struct FsStorageBackend {
    root: PathBuf,
}

impl FsStorageBackend {
    pub async fn new<P>(root: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let root = root.as_ref();
        tokio::fs::create_dir_all(root).await?;
        Ok(FsStorageBackend {
            root: root.to_path_buf(),
        })
    }

    async fn create_parents(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    fn resolve_path(&self, path: &StoragePath) -> PathBuf {
        self.root.join::<PathBuf>(path.as_path())
    }
}

#[async_trait::async_trait]
impl AbstractStorageBackend for FsStorageBackend {
    async fn open_writer(&self, path: &StoragePath) -> Result<Box<dyn AbstractStorageWriter>> {
        let path = self.resolve_path(path);
        self.create_parents(&path).await?;

        let file = File::create_new(&path).await?;
        let writer = BufWriter::with_capacity(32 * 1024, file);

        let writer = FsStorageWriter { writer, path };
        Ok(Box::new(writer))
    }

    async fn get_reader(&self, path: &StoragePath) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        let path = self.resolve_path(path);
        let file = File::open(path).await?;
        Ok(Box::new(file))
    }

    async fn rename(&self, from: &StoragePath, to: &StoragePath) -> Result<bool> {
        let from = self.resolve_path(from);
        let to = self.resolve_path(to);

        // TODO: Fix TOCTOU
        if tokio::fs::try_exists(&to).await? {
            return Ok(false);
        }

        self.create_parents(&to).await?;
        tokio::fs::rename(&from, &to).await?;

        Ok(true)
    }

    async fn remove(&self, path: &StoragePath) -> Result<bool> {
        let path = self.resolve_path(path);

        match tokio::fs::remove_file(path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
}

pub struct FsStorageWriter {
    writer: BufWriter<File>,
    path: PathBuf,
}

#[async_trait::async_trait]
impl AbstractStorageWriter for FsStorageWriter {
    async fn write_chunk(&mut self, data: bytes::Bytes) -> Result<()> {
        self.writer.write_all(&data).await?;
        Ok(())
    }

    async fn finalize(mut self: Box<Self>) -> Result<()> {
        self.writer.flush().await?;
        Ok(())
    }

    async fn abort(mut self: Box<Self>) -> Result<()> {
        drop(self.writer);
        tokio::fs::remove_file(&self.path).await?;
        Ok(())
    }
}
