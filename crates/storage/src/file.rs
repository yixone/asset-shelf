use mimetype::MimeType;
use result::Result;
use storage_backend::core::path::StoragePath;

use crate::Storage;

pub struct UnreleasedFile<'a> {
    /// The path where the unreleased file is stored
    pub(crate) temp_path: StoragePath,
    /// A storage area that holds this file
    pub(crate) owner: &'a Storage,

    /// The file to be released
    pub target: StorageFile,
}

impl<'a> UnreleasedFile<'a> {
    pub async fn release(self) -> Result<StorageFile> {
        if let Err(e) = self.owner.rename(&self.temp_path, &self.target.path).await {
            self.abort().await;
            return Err(e);
        }
        Ok(self.target)
    }

    pub async fn abort(self) {
        self.owner.remove_safely(&self.temp_path).await;
    }
}

#[derive(Debug)]
pub struct StorageFile {
    pub path: StoragePath,
    pub mimetype: MimeType,
    pub size_bytes: usize,
    pub elapsed_ms: u64,
}
