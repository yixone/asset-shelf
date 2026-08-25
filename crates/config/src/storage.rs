use serde::{Deserialize, Serialize};

const DEFAULT_STORAGE_DIR: &str = "storage/global";
const DEFAULT_STORAGE_TEMP: &str = "storage/temp";

const DEFAULT_MAX_SIZE_MB: usize = 1024;

/// File storage configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct StorageConfig {
    backend: StorageBackendConfig,
    max_file_size_mb: usize,
}

/// File storage backend configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendConfig {
    /// Configuration for native file storage
    Native { dir: String, temp: String },
}

impl StorageConfig {
    /// Returns a reference to the backend of this [`StorageConfig`]
    pub fn backend(&self) -> &StorageBackendConfig {
        &self.backend
    }

    /// Returns the maximum size of a file in storage in bytes
    pub fn max_size_bytes(&self) -> usize {
        DEFAULT_MAX_SIZE_MB * 1024 * 1024
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            backend: StorageBackendConfig::Native {
                dir: DEFAULT_STORAGE_DIR.to_string(),
                temp: DEFAULT_STORAGE_TEMP.to_string(),
            },
            max_file_size_mb: DEFAULT_MAX_SIZE_MB,
        }
    }
}
