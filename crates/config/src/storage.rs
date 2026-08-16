use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_STORAGE_PATH: &str = "storage/global";
const DEFAULT_STORAGE_TEMP: &str = "storage/temp";

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct StorageConfig {
    storage_root: String,
    temp_dir: String,
}

impl StorageConfig {
    pub fn root(&self) -> PathBuf {
        PathBuf::from(&self.storage_root)
    }

    pub fn temp(&self) -> PathBuf {
        PathBuf::from(&self.temp_dir)
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            storage_root: DEFAULT_STORAGE_PATH.into(),
            temp_dir: DEFAULT_STORAGE_TEMP.into(),
        }
    }
}
