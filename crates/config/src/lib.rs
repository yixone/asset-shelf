use std::{
    fs::File,
    io::{ErrorKind, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

mod database;
mod instance;
mod server;
mod storage;

pub use database::{DatabaseConfig, DatabaseDriverConfig};
pub use storage::{StorageBackendConfig, StorageConfig};

/// Configuration file header with additional information
const CONFIG_FILE_HEADER: &str =
    "# Read about application configuration: https://github.com/yixone/asset-shelf\n\n";

/// Application configuration container
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ApplicationConfig {
    pub instance: instance::InstanceConfig,
    pub server: server::ServerConfig,
    pub storage: storage::StorageConfig,
    pub database: database::DatabaseConfig,
}

impl ApplicationConfig {
    /// Deserializes the application configuration
    fn deserialize(data: &[u8]) -> Result<Self, ConfigError> {
        toml::from_slice(data).map_err(|_| ConfigError::DeserializingFailed)
    }

    /// Serializes the application configuration
    fn serialize(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(&self).map_err(|_| ConfigError::SerializingFailed)
    }

    /// Tries to load the application configuration. Otherwise, it creates a default file
    pub fn try_load(path: impl AsRef<Path>, create_if_missing: bool) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        match std::fs::read(path) {
            Ok(buf) => match ApplicationConfig::deserialize(&buf) {
                Ok(cfg) => Ok(cfg),
                Err(e) => {
                    tracing::error!("Failed to read config! Application launch aborted!");
                    Err(e)
                }
            },
            Err(e) if e.kind() == ErrorKind::NotFound => {
                if !create_if_missing {
                    return Err(ConfigError::FileDoesNotExist);
                }

                tracing::info!("Config not found; Using default configuration");

                let default = ApplicationConfig::default();

                let mut file = File::create(path).map_err(ConfigError::IoError)?;

                file.write_all(CONFIG_FILE_HEADER.as_bytes())
                    .map_err(ConfigError::IoError)?;

                let serialized = default.serialize()?;
                file.write_all(serialized.as_bytes())
                    .map_err(ConfigError::IoError)?;

                Ok(default)
            }
            Err(e) => Err(ConfigError::IoError(e)),
        }
    }
}

/// Application configuration error
#[derive(Debug)]
pub enum ConfigError {
    DeserializingFailed,
    SerializingFailed,
    FileDoesNotExist,
    IoError(std::io::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ConfigError {}
