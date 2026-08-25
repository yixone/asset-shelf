use std::{
    fs::File,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const MANIFEST_FILE_NAME: &str = "manifest";

/// Application Library Manifest
#[derive(Debug, Serialize, Deserialize)]
pub struct LibManifest {
    /// Library version
    lib_version: String,
    /// Library name
    lib_name: String,
    /// Library id
    lib_id: u8,
    /// Path to the library configuration file
    config_path: String,
    /// Selected file storage backend
    storage: LibStorage,
    /// Selected database driver
    database: LibDatabase,
}

impl LibManifest {
    /// Creates a new [`LibManifest`] builder
    pub fn builder() -> ManifestBuilder {
        ManifestBuilder::default()
    }

    /// Opens a [`LibManifest`] from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        match std::fs::read(path) {
            Ok(b) => {
                let deserialized = toml::from_slice::<LibManifest>(&b)
                    .map_err(|_| ManifestError::DeserializationFailed)?;

                Ok(deserialized)
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Err(ManifestError::FileNotFound),
            Err(e) => Err(ManifestError::Io(e)),
        }
    }

    /// Loades a [`LibManifest`] from the specified directory
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        Self::from_file(path)
    }

    /// Saves the manifest to the specified directory
    pub fn save_in_dir(&self, dir: impl AsRef<Path>) -> Result<(), ManifestError> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);

        let serialized = toml::to_string(&self).map_err(|_| ManifestError::SerializationFailed)?;

        let mut file = File::create(path).map_err(ManifestError::Io)?;
        file.write_all(serialized.as_bytes())
            .map_err(ManifestError::Io)?;

        Ok(())
    }

    /// Returns the config path of this [`LibManifest`]
    pub fn config_path(&self) -> PathBuf {
        PathBuf::from(&self.config_path)
    }

    /// Returns the storage of this [`LibManifest`]
    pub fn storage(&self) -> LibStorage {
        self.storage
    }

    /// Returns the database of this [`LibManifest`]
    pub fn database(&self) -> LibDatabase {
        self.database
    }

    /// Returns the lib id of this [`LibManifest`]
    pub fn lib_id(&self) -> u8 {
        self.lib_id
    }
}

/// Library file storage backend
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibStorage {
    /// Native file storage backend
    Native,
}

/// Library database driver
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibDatabase {
    /// Sqlite database driver
    Sqlite,
}

/// Manifest Builder Structure
pub struct ManifestBuilder {
    lib_version: String,
    lib_name: String,
    config_path: String,
    storage: LibStorage,
    database: LibDatabase,
}

impl ManifestBuilder {
    pub fn with_version(mut self, version: &str) -> Self {
        self.lib_version = version.to_string();
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.lib_name = name.to_string();
        self
    }

    pub fn with_config(mut self, path: &str) -> Self {
        self.config_path = path.to_string();
        self
    }

    pub fn with_storage(mut self, storage: LibStorage) -> Self {
        self.storage = storage;
        self
    }

    pub fn with_database(mut self, database: LibDatabase) -> Self {
        self.database = database;
        self
    }

    pub fn build(self, id: u8) -> LibManifest {
        LibManifest {
            lib_version: self.lib_version,
            lib_name: self.lib_name,
            lib_id: id,
            config_path: self.config_path,
            storage: self.storage,
            database: self.database,
        }
    }
}

impl Default for ManifestBuilder {
    fn default() -> Self {
        ManifestBuilder {
            lib_version: env!("CARGO_PKG_VERSION").to_string(),
            lib_name: "unnamed".to_string(),
            config_path: "config.toml".to_string(),
            storage: LibStorage::Native,
            database: LibDatabase::Sqlite,
        }
    }
}

/// Error interacting with manifest
#[derive(Debug)]
pub enum ManifestError {
    /// Manifest deserialization error
    DeserializationFailed,

    /// Manifest serialization error
    SerializationFailed,

    /// The manifest file at the specified path was not found
    FileNotFound,

    Io(std::io::Error),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ManifestError {}
