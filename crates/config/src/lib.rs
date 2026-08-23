use std::{
    fs::File,
    io::{ErrorKind, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

pub mod database;
pub mod instance;
pub mod server;
pub mod storage;

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ApplicationConfig {
    pub instance: instance::InstanceConfig,
    pub server: server::ServerConfig,
    pub storage: storage::StorageConfig,
    pub database: database::DatabaseConfig,
}

impl ApplicationConfig {
    pub fn deserialize(data: &[u8]) -> Result<Self, ConfigError> {
        let deserialized = toml::from_slice(data).map_err(|_| ConfigError::DeserializingFailed)?;
        Ok(deserialized)
    }

    pub fn serialize(&self) -> Result<String, ConfigError> {
        let serialized =
            toml::to_string_pretty(&self).map_err(|_| ConfigError::SerializingFailed)?;
        Ok(serialized)
    }

    pub fn try_load(path: impl AsRef<Path>, create_if_missing: bool) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        match std::fs::read(path) {
            Ok(buf) => match ApplicationConfig::deserialize(&buf) {
                Ok(cfg) => Ok(cfg),
                Err(e) => {
                    tracing::error!("Failed to deserialize config! Application launch aborted!");
                    Err(e)
                }
            },
            Err(e) if e.kind() == ErrorKind::NotFound => {
                if !create_if_missing {
                    return Err(ConfigError::FileDoesNotExist);
                }

                let default = ApplicationConfig::default();

                let serialized = default.serialize()?;
                let mut file = File::create(path).map_err(ConfigError::IoError)?;
                file.write_all(serialized.as_bytes())
                    .map_err(ConfigError::IoError)?;

                Ok(default)
            }
            Err(e) => Err(ConfigError::IoError(e)),
        }
    }
}

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
