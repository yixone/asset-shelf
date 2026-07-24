use mimetype::MimeType;
use storage_backend::core::path::StoragePath;

use crate::shard_key;

#[derive(Debug)]
pub struct UploadedFile {
    pub mimetype: MimeType,
    pub size_bytes: usize,
    pub elapsed_ms: u64,
}

#[derive(Debug)]
pub struct StorageUploadResult {
    pub path: StoragePath,
    pub file: UploadedFile,
}

#[derive(Debug)]
pub struct TempStorageFile {
    pub key: String,
    pub file: UploadedFile,
}

impl TempStorageFile {
    pub fn commit_path(&self, namespace: &str) -> StoragePath {
        StoragePath {
            namespace: namespace.to_string(),
            key: shard_key(&self.key, 2),
        }
    }
}
