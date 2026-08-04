use std::path::PathBuf;

use flake_id::FlakeIdGenerator;
use futures::TryStreamExt;
use result::{Result, error::ResultExt};
use tokio::io::AsyncRead;
use tokio_util::io::ReaderStream;

use crate::{
    backend::{StorageBackend, fs::NativeFsStorageBackend},
    files::{LocalFile, ReservedFile, UncommitedFile},
    global::{GlobalPathData, GlobalSection, generate_global_path},
    temp::TempSection,
};

pub mod backend;
pub mod file;

pub mod files;

pub mod global;
pub mod temp;

pub use backend::path::StoragePath;

/// File storage divided into two sections:
/// - `Temporary Storage`
/// - `Global (Persistent) Storage`
pub struct Storage {
    /// Section for storing persistent files
    global: GlobalSection,

    /// Section for storing temporary files
    temp: TempSection,
}

impl Storage {
    /// Creates a new [`Storage`]
    pub async fn new<B: StorageBackend + 'static>(
        backend: B,
        id_gen: FlakeIdGenerator,
        temp_dir: PathBuf,
    ) -> Result<Storage> {
        Ok(Storage {
            global: GlobalSection {
                backend: Box::new(backend),
            },
            temp: TempSection {
                backend: NativeFsStorageBackend::new(temp_dir).await?,
                temp_id_generator: id_gen,
            },
        })
    }

    /// Uploads a file to the temporary section of the storage
    /// and returns a "pointer" for committing it
    pub async fn upload<D, F>(
        &self,
        path: GlobalPathData<'_>,
        data: D,
        mut chunk_callback: F,
    ) -> Result<UncommitedFile<'_>>
    where
        D: AsyncRead + Unpin,
        F: FnMut(&[u8]) -> Result<()>,
    {
        // Generates the future file path in the storage
        let path = generate_global_path(path);

        // Creates a file size counter
        let mut size_bytes = 0;

        // Creates a reader stream for reading data
        let mut data_reader = ReaderStream::with_capacity(data, 32 * 1024);

        // Creates a write operation for a file in the temporary section of the storage
        let temp_path = self.temp.generate_temp_path();
        let mut blob_writer = self.temp.backend.create(&temp_path).await?;

        // Reads the incoming stream
        while let Some(chunk) = data_reader.try_next().await.to_app_err()? {
            // Invokes a callback for each chunk,
            // allowing the calling code to process and validate data on the fly
            chunk_callback(&chunk)?;

            size_bytes += chunk.len();
            blob_writer.write(chunk).await?;
        }
        blob_writer.flush().await?;

        // Returns an uncommitted file.
        Ok(UncommitedFile {
            owner: self,
            size_bytes,
            temp_path,
            global_path: path,
            need_cleanup: true,
        })
    }

    /// Moves a file from the temporary section of the storage to the global section,
    /// removing it from the temporary section, and returns the number of bytes moved
    pub(crate) async fn move_to_global_section(
        &self,
        temp: &StoragePath,
        global: &StoragePath,
    ) -> Result<usize> {
        // Copies a file from the temporary section to the permanent section
        let bytes_moved = self
            .copy_to_section((&self.temp.backend, temp), (&*self.global.backend, global))
            .await?;

        // Deletes the file from the temporary section
        if let Err(e) = self.temp.backend.remove(temp).await {
            tracing::warn!(err = ?e, "Failed to delete temp file after moving to global storage section");
        }

        Ok(bytes_moved)
    }

    /// Copies the file to the specified section and and returns the number of bytes moved
    ///
    /// ### Note:
    /// It might be worth revisiting the approach to moving
    /// files between sections in the future,
    /// but for now, moving a file for instance,
    /// from `FS` to `S3`-is implemented solely via copying.
    async fn copy_to_section(
        &self,
        from: (&dyn StorageBackend, &StoragePath),
        to: (&dyn StorageBackend, &StoragePath),
    ) -> Result<usize> {
        // Obtains a reader for the file from the `from` section
        let from_reader = from.0.read(from.1).await?;

        // Creates a writer for the file in the `to` section
        let mut to_writer = to.0.create(to.1).await?;

        // Creates a reader stream for reading `from` data
        let mut data_reader = ReaderStream::with_capacity(from_reader, 32 * 1024);
        let mut bytes_transfered = 0;

        // Copying chunks
        while let Some(chunk) = data_reader.try_next().await.to_app_err()? {
            bytes_transfered += chunk.len();
            to_writer.write(chunk).await?;
        }

        // Writer confirmation and closing
        to_writer.flush().await?;

        Ok(bytes_transfered)
    }

    /// Returns a reader for a file from the global section of the storage
    pub async fn open(&self, path: &StoragePath) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        let reader = self.global.backend.read(path).await?;
        Ok(reader)
    }

    /// Returns the reader to the specified byte range of the file within the global storage section
    pub async fn open_ranged(
        &self,
        path: &StoragePath,
        start: u64,
        end: Option<u64>,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        let reader = self.global.backend.read_ranged(path, start, end).await?;
        Ok(reader)
    }

    /// Copies the specified file from the global storage to a temporary location and returns it
    pub async fn open_local(&self, path: &StoragePath) -> Result<LocalFile> {
        // Generates a temporary path for a file
        let temp_path = self.temp.generate_temp_path();

        // Copies a file from the global section
        self.copy_to_section(
            (&*self.global.backend, path),
            (&self.temp.backend, &temp_path),
        )
        .await?;

        // Returns local file data
        let local_file = LocalFile {
            path: self.temp.resolve_path(&temp_path),
            need_cleanup: true,
        };
        Ok(local_file)
    }

    /// Reserves a path for a file in temporary storage,
    /// allowing the file to be created, written, and edited outside the main storage,
    /// and subsequently enabling the published temporary file to be moved to the global storage section
    pub fn reserve(&self, path: GlobalPathData<'_>) -> ReservedFile<'_> {
        // Generates a reserved temp path
        let temp_path = self.temp.generate_temp_path();
        let real_path = self.temp.resolve_path(&temp_path);

        // Generates the future file path in the storage
        let path = generate_global_path(path);

        ReservedFile {
            owner: self,
            publish_path: path,
            temp_path,
            path: real_path,
            need_cleanup: true,
        }
    }

    /// Deletes a file from the global section of the storage, ignoring any errors that occur
    ///
    /// Returns `true` if the file was deleted, and `false` otherwise
    pub async fn remove_safely(&self, path: &StoragePath) -> bool {
        // Deletes the file from the section
        if let Err(e) = self.global.backend.remove(path).await {
            tracing::warn!(err = ?e, "Failed to delete file from global section");
            return false;
        }
        true
    }
}
