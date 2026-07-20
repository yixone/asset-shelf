use mimetype::MimeType;
use storage_backend::core::path::StoragePath;

#[derive(Debug)]
pub struct StorageUploadResult {
    pub path: StoragePath,
    pub mimetype: MimeType,
    pub size_bytes: usize,
    // TODO: MOVE TO UploadStats
    pub elapsed_ms: u64,
}
