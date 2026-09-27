use std::{
    path::{Path, PathBuf},
    pin::pin,
};

use tokio::{
    fs::File,
    io::{AsyncRead, AsyncSeek, AsyncWriteExt, BufReader, BufWriter},
};

use crate::{
    fs::{dir, rename_exclusive},
    utils::futures::asyncify,
};

/// A file staged for atomic publication
///
/// A staged file is written to a temporary path and can be published
/// at its target path using [`StagedFile::commit`]. Dropping an uncommitted
/// staged file removes its temporary file
pub struct StagedFile {
    pub(crate) temp_path: PathBuf,
    pub(crate) target_path: PathBuf,
    remove_on_drop: bool,
    size_bytes: u64,
}

impl StagedFile {
    /// Creates a new [`StagedFile`]
    pub fn new(temp_path: PathBuf, target_path: PathBuf, remove_on_drop: bool) -> Self {
        Self {
            temp_path,
            target_path,
            remove_on_drop,
            size_bytes: 0,
        }
    }

    /// Atomically publishes the staged file at its target path
    ///
    /// The operation fails if the target already exists. On failure, the
    /// staged file remains owned by the operation and is cleaned up on drop
    pub async fn commit(mut self) -> std::io::Result<()> {
        let from = self.temp_path.clone();
        let to = self.target_path.clone();

        dir::create_parents(&to).await?;

        asyncify(move || rename_exclusive(from, to)).await??;
        self.remove_on_drop = false;
        Ok(())
    }

    /// Aborts the staged file and removes its temporary file
    ///
    /// Aborting an already removed temporary file succeeds without error
    pub async fn abort(mut self) -> std::io::Result<()> {
        match tokio::fs::remove_file(&self.temp_path).await {
            Ok(_) => {
                self.remove_on_drop = false;
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.remove_on_drop = false;
                Ok(())
            }
            Err(e) => {
                tracing::error!(err = ?e, "Failed to delete staged file on abort");
                Err(e)
            }
        }
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if !self.remove_on_drop {
            return;
        }

        if let Err(e) = std::fs::remove_file(&self.temp_path)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            tracing::error!(err = ?e, "Failed to delete staged file on abort");
        }
    }
}

/// Writes data to a [`StagedFile`]
///
/// The staged file is owned by the writer until [`FileWriter::finish`] is
/// called or the writer is dropped
pub struct FileWriter {
    staged: StagedFile,
    writer: BufWriter<File>,
    size_bytes: u64,
}

impl FileWriter {
    /// Creates a new [`FileWriter`]
    pub fn new(staged: StagedFile, writer: BufWriter<File>) -> Self {
        Self {
            staged,
            writer,
            size_bytes: 0,
        }
    }

    /// Writes data to the staged file
    pub async fn write(&mut self, data: impl AsRef<[u8]>) -> std::io::Result<()> {
        self._write(data.as_ref()).await
    }

    async fn _write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data).await?;
        self.size_bytes += data.len() as u64;
        Ok(())
    }

    /// Finishes writing and returns the staged file
    ///
    /// The buffered data is flushed before the staged file is returned
    pub async fn finish(mut self) -> std::io::Result<StagedFile> {
        self.writer.flush().await?;
        self.staged.size_bytes = self.size_bytes;
        Ok(self.staged)
    }

    /// Aborts the writer and removes its temporary file
    pub async fn abort(self) -> std::io::Result<()> {
        self.staged.abort().await
    }
}

/// Provides a temporary path for uploading a staged file using an external
/// process or another foreign writer
///
/// The staged file is owned by the uploader until [`ForeignUploader::finish`] is
/// called or the uploader is dropped
pub struct ForeignUploader {
    staged: StagedFile,
}

impl ForeignUploader {
    /// Creates a new [`ForeignUploader`]
    pub fn new(staged: StagedFile) -> Self {
        Self { staged }
    }

    /// Returns the temporary path where the foreign writer should store data
    pub fn path(&self) -> &Path {
        &self.staged.temp_path
    }

    /// Finishes the upload and returns the staged file
    pub async fn finish(mut self) -> std::io::Result<StagedFile> {
        let meta = tokio::fs::metadata(&self.staged.temp_path).await?;
        self.staged.size_bytes = meta.len();
        Ok(self.staged)
    }

    /// Aborts the uploader and removes its temporary file
    pub async fn abort(self) -> std::io::Result<()> {
        self.staged.abort().await
    }
}

/// Provides access to a locally stored file
pub struct LocalFile {
    path: PathBuf,
    len: u64,
}

impl LocalFile {
    /// Creates a new [`LocalFile`]
    pub fn new(path: PathBuf, len: u64) -> Self {
        Self { path, len }
    }

    /// Returns the local filesystem path of this file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the length of the file
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns `true` if the file is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Wrapper for a stored file reader
pub struct FileReader {
    reader: BufReader<File>,
    len: u64,
}

impl FileReader {
    /// Creates a new [`FileReader`]
    pub fn new(reader: BufReader<File>, len: u64) -> Self {
        Self { reader, len }
    }

    /// Returns the length of the file when the reader was created
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns `true` if the file is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AsyncRead for FileReader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        pin!(&mut self.reader).poll_read(cx, buf)
    }
}

impl AsyncSeek for FileReader {
    fn poll_complete(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        pin!(&mut self.reader).poll_complete(cx)
    }

    fn start_seek(
        mut self: std::pin::Pin<&mut Self>,
        position: std::io::SeekFrom,
    ) -> std::io::Result<()> {
        pin!(&mut self.reader).start_seek(position)
    }
}
