use serde::{Deserialize, Serialize};

const DEFAULT_DATABASE_PATH: &str = "storage/data.db";

/// Application database configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// Database driver configuration
    driver: DatabaseDriverConfig,
}

impl DatabaseConfig {
    /// Returns a reference to the driver of this [`DatabaseConfig`]
    pub fn driver(&self) -> &DatabaseDriverConfig {
        &self.driver
    }
}

/// Database driver configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseDriverConfig {
    /// Sqlite driver
    Sqlite { path: String },
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            driver: DatabaseDriverConfig::Sqlite {
                path: DEFAULT_DATABASE_PATH.to_string(),
            },
        }
    }
}
