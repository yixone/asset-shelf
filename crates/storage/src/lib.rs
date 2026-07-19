use std::time::Instant;

use flake_id::{FlakeIdGenerator, FlakeIdHex};
use futures::TryStreamExt;
use mimetype::MimeType;
use storage_backend::{
    StorageBackend,
    core::{path::StoragePath, result::StorageBackendError},
};
use tokio::io::AsyncRead;
use tokio_util::io::ReaderStream;

#[derive(Debug)]
pub enum StorageError {
    BackendErr(StorageBackendError),
    FileTooLarge { received: usize, excepted: usize },
    UnsuppotedMimetype,
}

impl<E> From<E> for StorageError
where
    StorageBackendError: From<E>,
{
    fn from(err: E) -> Self {
        StorageError::BackendErr(err.into())
    }
}

pub struct Storage {
    backend: StorageBackend,
    id_gen: FlakeIdGenerator,
    max_size: usize,
}

#[derive(Debug)]
pub struct StorageUploadResult {
    path: StoragePath,
    mimetype: MimeType,
    size_bytes: usize,
    // TODO: MOVE TO UploadStats
    elapsed_ms: u64,
}

const TEMP_NAMESPACE: &str = "temp";

impl Storage {
    /// Creates a new [`Storage`]
    pub fn new(backend: StorageBackend, id_gen: FlakeIdGenerator, max_size: usize) -> Storage {
        Storage {
            backend,
            id_gen,
            max_size,
        }
    }

    fn generate_key(&self) -> String {
        self.id_gen.generate_as::<FlakeIdHex>().to_string()
    }

    pub async fn upload<D>(
        &self,
        namespace: &str,
        data: D,
    ) -> Result<StorageUploadResult, StorageError>
    where
        D: AsyncRead + Unpin,
    {
        let start_time = Instant::now();
        let key = self.generate_key();

        let temp_path = StoragePath::new(TEMP_NAMESPACE.into(), key.clone());
        let mut size_bytes = 0;
        let mut mimetype = None;

        let mut data_reader = ReaderStream::with_capacity(data, 32 * 1024);
        let mut blob_writer = self.backend.open_writer(&temp_path).await?;

        let mut header_buffer = Vec::with_capacity(128);
        let header_cap = header_buffer.capacity();

        while let Some(chunk) = data_reader.try_next().await? {
            size_bytes += chunk.len();

            if size_bytes > self.max_size {
                blob_writer.abort().await?;
                return Err(StorageError::FileTooLarge {
                    received: size_bytes,
                    excepted: self.max_size,
                });
            }

            let header_len = header_buffer.len();
            if header_len < header_cap {
                let h_chunk = &chunk[..chunk.len().min(header_cap - header_len)];
                header_buffer.extend_from_slice(h_chunk);
            }
            if mimetype.is_none() && header_len >= header_cap {
                mimetype = match MimeType::guess(&header_buffer) {
                    Ok(m) => Some(m),
                    Err(_) => {
                        blob_writer.abort().await?;
                        return Err(StorageError::UnsuppotedMimetype);
                    }
                };
            }

            blob_writer.write_chunk(chunk).await?;
        }
        let Some(mimetype) = mimetype else {
            blob_writer.abort().await?;
            return Err(StorageError::UnsuppotedMimetype);
        };
        blob_writer.finalize().await?;

        let key = shard_key(&key, 2);

        let dest = StoragePath {
            namespace: namespace.into(),
            key,
        };
        self.backend.rename(&temp_path, &dest).await?;

        let res = StorageUploadResult {
            path: dest,
            mimetype,
            size_bytes,
            elapsed_ms: (Instant::now() - start_time).as_millis() as u64,
        };

        Ok(res)
    }

    pub async fn get(
        &self,
        path: &StoragePath,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, StorageError> {
        let reader = self.backend.get_reader(path).await?;
        Ok(reader)
    }

    pub async fn remove(&self, path: &StoragePath) -> Result<bool, StorageError> {
        let res = self.backend.remove(path).await?;
        Ok(res)
    }
}

fn shard_key(key: &str, levels: usize) -> String {
    let mut res = String::new();

    for i in 0..levels {
        let idx = 2 * i;
        if idx + 2 > key.len() {
            break;
        }
        res.push_str(&key[idx..idx + 2]);
        res.push('/');
    }

    res.push_str(key);
    res
}
