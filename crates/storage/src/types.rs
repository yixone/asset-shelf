use mimetype::MimeType;
use storage_backend::core::path::StoragePath;

#[derive(Debug)]
pub struct UncommittedFile {
    /// The path where the uncommitted file is stored
    pub(crate) temp_path: StoragePath,

    /// The file to be committed
    pub target: StorageFile,
}

#[derive(Debug)]
pub struct StorageFile {
    pub path: StoragePath,
    pub mimetype: MimeType,
    pub size_bytes: usize,
    pub elapsed_ms: u64,
}
