use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_DATABASE_PATH: &str = "storage/data.db";

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DatabaseConfig {
    database_path: String,
}

impl DatabaseConfig {
    /// Returns the database path of this [`DatabaseConfig`]
    pub fn path(&self) -> PathBuf {
        PathBuf::from(&self.database_path)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            database_path: DEFAULT_DATABASE_PATH.to_string(),
        }
    }
}
