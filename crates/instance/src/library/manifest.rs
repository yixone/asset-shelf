use std::{
    fs::File,
    io::{ErrorKind, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::library::LibraryError;

const MANIFEST_FILE_NAME: &str = "manifest";

const MANIFEST_FILE_HEADER: &str =
    "# DO NOT MANUALLY CHANGE MANIFEST PARAMETERS!\n# THIS MAY CAUSE UNEXPECTED ERRORS!\n";

/// Application Library Manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibManifest {
    /// Library metadata
    library: LibMeta,
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

    /// Loades a [`LibManifest`] from the specified directory
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self, LibraryError> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        match std::fs::read(path) {
            Ok(b) => {
                let deserialized = toml::from_slice::<LibManifest>(&b)
                    .map_err(|_| LibraryError::DeserializationFailed)?;

                Ok(deserialized)
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Err(LibraryError::ManifestNotFound),
            Err(e) => Err(LibraryError::Io(e)),
        }
    }

    /// Saves the manifest to the specified directory
    pub fn save_in_dir(&self, dir: impl AsRef<Path>) -> Result<(), LibraryError> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);

        let serialized = toml::to_string(&self).map_err(|_| LibraryError::SerializationFailed)?;

        let mut file = File::create(path).map_err(LibraryError::Io)?;
        file.write_all(MANIFEST_FILE_HEADER.as_bytes())
            .map_err(LibraryError::Io)?;
        file.write_all(serialized.as_bytes())
            .map_err(LibraryError::Io)?;

        Ok(())
    }

    /// Returns the storage of this [`LibManifest`]
    pub fn storage(&self) -> &LibStorage {
        &self.storage
    }

    /// Returns the database of this [`LibManifest`]
    pub fn database(&self) -> &LibDatabase {
        &self.database
    }

    /// Returns the lib id of this [`LibManifest`]
    pub fn lib_id(&self) -> u8 {
        self.library.node_id
    }

    /// Returns a reference to the name of this [`LibManifest`]
    pub fn name(&self) -> &str {
        &self.library.name
    }
}

/// Application Library metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibMeta {
    /// Library version
    version: String,
    /// Library name
    name: String,
    /// Library node id
    node_id: u8,
}

/// Library file storage backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibStorage {
    /// Native file storage backend
    Native { dir: String, temp: String },
}

/// Library database driver
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibDatabase {
    /// Sqlite database driver
    Sqlite { path: String },
}

/// Manifest Builder Structure
pub struct ManifestBuilder {
    lib_version: String,
    lib_name: String,
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
            library: LibMeta {
                version: self.lib_version,
                name: self.lib_name,
                node_id: id,
            },
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
            storage: LibStorage::Native {
                dir: "global".to_string(),
                temp: "temp".to_string(),
            },
            database: LibDatabase::Sqlite {
                path: "data.db".to_string(),
            },
        }
    }
}
