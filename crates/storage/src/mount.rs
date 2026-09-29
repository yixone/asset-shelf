use std::path::PathBuf;

use storage_types::DiskUsageStats;

use crate::{fs::statvfs, utils::futures::asyncify};

/// A physical root directory of the storage
///
/// The storage root defines the root directory under which all storage files
/// are physically stored
pub struct StorageMountPoint {
    path: PathBuf,
}

impl StorageMountPoint {
    /// Creates a new [`StorageMountPoint`]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Returns disk usage stats for the fs containing the storage root
    pub async fn disk_usage(&self) -> std::io::Result<DiskUsageStats> {
        let path = self.path.clone();
        asyncify(move || statvfs(path)).await?
    }

    /// Resolves a relative storage path against the storage root
    pub fn resolve_path(&self, path: impl AsRef<str>) -> PathBuf {
        self.path.join(path.as_ref())
    }

    /// Returns `true` if the storage root exists
    pub async fn exists(&self) -> std::io::Result<bool> {
        tokio::fs::try_exists(&self.path).await
    }

    /// Creates the storage root and all missing parent directories
    pub async fn ensure(&self) -> std::io::Result<()> {
        crate::fs::dir::create_parents(&self.path).await
    }

    /// Returns a reference to the path of this [`StorageMountPoint`]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
