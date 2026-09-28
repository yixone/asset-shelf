use std::{io::SeekFrom, path::PathBuf};

use tokio::{
    fs::File,
    io::{AsyncSeekExt, BufReader, BufWriter},
};

use crate::{
    file::{FileReader, FileWriter, ForeignUploader, LocalFile, StagedFile},
    fs::{dir, rename_exclusive},
    mount::StorageMountPoint,
    path::{StoragePath, validate_path},
    result::{Result, StorageError},
    types::{DiskUsageStats, FileMetadata},
    utils::futures::asyncify,
};

/// Managed file storage
///
/// A [`Storage`] manages files under a single [`StorageMountPoint`].
/// Files are addressed using logical [`StoragePath`] values rather than
/// physical filesystem paths
///
/// New files are first written to a temporary staging path and published
/// using an [exclusive rename](rename_exclusive). This prevents incomplete
/// files from being published at its target path
pub struct Storage {
    root: StorageMountPoint,
}

impl Storage {
    /// Creates a new [`Storage`]
    pub fn new(root: StorageMountPoint) -> Self {
        Storage { root }
    }

    /// Resolves a logical storage path to its physical filesystem path
    #[inline]
    fn realpath(&self, path: impl AsRef<StoragePath>) -> PathBuf {
        self.root.resolve_path(path.as_ref())
    }

    /// Creates a writer for a new staged file
    ///
    /// The target path must not already exist. The returned [`FileWriter`]
    /// owns the staged file until [`FileWriter::finish`] or [`FileWriter::abort`] is called
    ///
    /// The file is not published until the resulting [`StagedFile`] is committed
    pub async fn create_file_writer(&self, path: impl AsRef<StoragePath>) -> Result<FileWriter> {
        let staged = self.stage(&path).await?;

        let file = File::create_new(&staged.temp_path).await?;
        let writer = BufWriter::with_capacity(32 * 1024, file);

        let out = FileWriter::new(staged, writer);
        Ok(out)
    }

    /// Creates a foreign uploader for a new staged file
    ///
    /// The returned [`ForeignUploader`] provides a temporary filesystem path
    /// where an external process or another writer can create the file
    ///
    /// The target path must not already exists. The file is not published
    /// until the resulting [`StagedFile`] is committed
    pub async fn create_file_uploader(
        &self,
        path: impl AsRef<StoragePath>,
    ) -> Result<ForeignUploader> {
        let staged = self.stage(&path).await?;

        let out = ForeignUploader::new(staged);
        Ok(out)
    }

    /// Creates a staged file for the specified storage path
    ///
    /// The target path must not already exists
    async fn stage(&self, path: impl AsRef<StoragePath>) -> Result<StagedFile> {
        let path = path.as_ref();
        validate_path(path)?;

        let temp_name = path
            .file_name()
            .map(|n| format!("~{}-{}", n, base62::random_str(6)))
            .ok_or(StorageError::InvalidPath)?;

        let mut temp_path = self.realpath(path);
        temp_path.set_file_name(temp_name);

        dir::create_parents(&temp_path).await?;

        let target_path = self.realpath(path);

        if tokio::fs::try_exists(&target_path).await? {
            return Err(StorageError::AlreadyExists);
        }

        let stage = StagedFile::new(temp_path, target_path, true);
        Ok(stage)
    }

    /// Opens a locally stored file
    ///
    /// Returns a [`LocalFile`] containing the physical filesystem path of the
    /// stored file
    pub async fn local_file(&self, path: impl AsRef<StoragePath>) -> Result<LocalFile> {
        validate_path(&path)?;
        let path = self.realpath(path);

        let meta = tokio::fs::metadata(&path).await?;

        let local = LocalFile::new(path, meta.len());
        Ok(local)
    }

    /// Opens a stored file for reading
    ///
    /// The reader starts at the beginning of the file and reads until EOF
    pub async fn reader(&self, path: impl AsRef<StoragePath>) -> Result<FileReader> {
        self.reader_seek(path, 0, None).await
    }

    /// Opens a stored file for reading from a specified byte range
    ///
    /// `start` specifies the first byte to read and `end`, when specified,
    /// specifies the last byte to read inclusively
    ///
    /// For example, `start = 10` and `end = Some(19)` reads bytes `10..=19`
    ///
    /// The requested range must be within the file bounds
    pub async fn reader_seek(
        &self,
        path: impl AsRef<StoragePath>,
        start: u64,
        end: Option<u64>,
    ) -> Result<FileReader> {
        validate_path(&path)?;
        let path = self.realpath(path);

        let mut file = File::open(&path).await?;

        let meta = tokio::fs::metadata(&path).await?;
        let len = meta.len();

        if start > len {
            return Err(StorageError::InvalidSeekRange);
        }

        file.seek(SeekFrom::Start(start)).await?;

        let buf_reader = BufReader::with_capacity(32 * 1024, file);

        let reader = match end {
            Some(end) => {
                if end < start || end > len {
                    return Err(StorageError::InvalidSeekRange);
                }

                let to_read = end.saturating_sub(start) + 1;
                FileReader::new(buf_reader, to_read)
            }
            None => FileReader::new(buf_reader, len - start),
        };

        Ok(reader)
    }

    /// Returns metadata for a stored file
    ///
    /// The returned metadata includes the file length and modification time
    pub async fn meta(&self, path: impl AsRef<StoragePath>) -> Result<FileMetadata> {
        validate_path(&path)?;
        let path = self.realpath(path);

        let meta = tokio::fs::metadata(&path).await?;

        let metadata = FileMetadata::new(meta.len(), meta.modified().ok());

        Ok(metadata)
    }

    /// Returns whether a file exists at the specified storage path
    ///
    /// Returns `false` if the path does not exists
    pub async fn exists(&self, path: impl AsRef<StoragePath>) -> Result<bool> {
        validate_path(&path)?;
        let path = self.realpath(path);

        let exists = tokio::fs::try_exists(path).await?;

        Ok(exists)
    }

    /// Returns disk usage statistics for the storage filesystem
    ///
    /// The statistics describe the filesystem containing the storage root
    pub async fn disk_usage(&self) -> Result<DiskUsageStats> {
        self.root.disk_usage().await.map_err(StorageError::from)
    }

    /// Renames a stored file to another storage path
    ///
    /// The destination must not already exists. Missing parent directories of the
    /// destination are created automatically
    pub async fn rename(
        &self,
        path: impl AsRef<StoragePath>,
        dest: impl AsRef<StoragePath>,
    ) -> Result<()> {
        validate_path(&path)?;
        validate_path(&dest)?;

        let from = self.realpath(path);
        let to = self.realpath(dest);

        dir::create_parents(&to).await?;
        asyncify(move || rename_exclusive(from, to)).await??;

        Ok(())
    }

    /// Removes a stored file
    ///
    /// Returns `true` if the file was removed and `false` if it did not exist
    ///
    /// Empty parent directories are removed after successful deletion when
    /// they can be safely removed
    pub async fn remove(&self, path: impl AsRef<StoragePath>) -> Result<bool> {
        validate_path(&path)?;
        let path = self.realpath(path);

        match tokio::fs::remove_file(&path).await {
            Ok(_) => {
                tracing::info!(
                    path = ?path,
                    "File removed"
                );
                dir::delete_parents_safely(self.root.path(), &path).await;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(StorageError::Io(e)),
        }
    }
}
