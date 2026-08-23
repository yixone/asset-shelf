use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_STORAGE_DIR: &str = "storage/global";
const DEFAULT_STORAGE_TEMP: &str = "storage/temp";

const DEFAULT_MAX_SIZE_MB: usize = 1024;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct StorageConfig {
    storage_dir: String,
    temp_dir: String,

    max_size_mb: usize,
}

impl StorageConfig {
    pub fn dir(&self) -> PathBuf {
        PathBuf::from(&self.storage_dir)
    }

    pub fn temp(&self) -> PathBuf {
        PathBuf::from(&self.temp_dir)
    }

    pub fn max_size_bytes(&self) -> usize {
        DEFAULT_MAX_SIZE_MB * 1024 * 1024
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            storage_dir: DEFAULT_STORAGE_DIR.into(),
            temp_dir: DEFAULT_STORAGE_TEMP.into(),
            max_size_mb: DEFAULT_MAX_SIZE_MB,
        }
    }
}
