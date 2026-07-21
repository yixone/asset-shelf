use std::time::Instant;

use flake_id::{FlakeIdGenerator, FlakeIdHex};
use futures::TryStreamExt;
use mimetype::MimeType;
use result::{Result, create_error, error::ResultExt};
use storage_backend::{StorageBackend, core::path::StoragePath};
use tokio::io::AsyncRead;
use tokio_util::io::ReaderStream;

use crate::types::StorageUploadResult;

pub mod types;

const TEMP_NAMESPACE: &str = "temp";
const SHARDING_LEVEL: usize = 2;

pub struct Storage {
    backend: StorageBackend,
    id_gen: FlakeIdGenerator,
    max_size: usize,
}

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

    async fn remove_safely(&self, path: &StoragePath) {
        // TODO: ADD ERROR TRACING!
        let _ = self.backend.remove(path).await;
    }

    pub async fn upload<D>(&self, namespace: &str, data: D) -> Result<StorageUploadResult>
    where
        D: AsyncRead + Unpin,
    {
        let start_time = Instant::now();
        let key = self.generate_key();

        let temp_path = StoragePath::new(TEMP_NAMESPACE.into(), key.clone());
        let mut size_bytes = 0;

        let mut data_reader = ReaderStream::with_capacity(data, 32 * 1024);
        let mut blob_writer = self.backend.open_writer(&temp_path).await?;

        let mut header_buf = Vec::with_capacity(128);
        let header_cap = header_buf.capacity();

        while let Some(chunk) = data_reader.try_next().await.to_app_err()? {
            size_bytes += chunk.len();

            if size_bytes > self.max_size {
                blob_writer.abort().await?;
                return Err(create_error!(FileTooLarge {
                    max_size: self.max_size
                }));
            }

            let header_len = header_buf.len();
            if header_len < header_cap {
                let h_chunk = &chunk[..chunk.len().min(header_cap - header_len)];
                header_buf.extend_from_slice(h_chunk);
            }
            blob_writer.write_chunk(chunk).await?;
        }

        let mimetype = match MimeType::guess(&header_buf) {
            Ok(m) => m,
            Err(_) => {
                blob_writer.abort().await?;
                return Err(create_error!(UnsupportedFileType));
            }
        };

        blob_writer.finalize().await?;

        let key = shard_key(&key, SHARDING_LEVEL);

        let dest = StoragePath {
            namespace: namespace.into(),
            key,
        };

        if !self.backend.rename(&temp_path, &dest).await? {
            self.remove_safely(&temp_path).await;
            return Err(create_error!(AlreadyExists));
        }

        let res = StorageUploadResult {
            path: dest,
            mimetype,
            size_bytes,
            elapsed_ms: start_time.elapsed().as_millis() as u64,
        };

        Ok(res)
    }

    pub async fn get(&self, path: &StoragePath) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        let reader = self.backend.get_reader(path).await?;
        Ok(reader)
    }

    pub async fn remove(&self, path: &StoragePath) -> Result<()> {
        self.remove_safely(path).await;
        Ok(())
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
