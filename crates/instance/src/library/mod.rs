use std::path::{Path, PathBuf};

pub mod manifest;
pub use manifest::{LibDatabase, LibManifest, LibStorage};

/// Library model
#[derive(Debug, Clone)]
pub struct Library {
    /// Library directory
    pub dir: PathBuf,
    /// Library manifest
    pub manifest: LibManifest,
}

impl Library {
    /// Creates a new [`Library`]
    pub fn new(dir: PathBuf, manifest: LibManifest) -> Self {
        Self { dir, manifest }
    }

    /// Saves the library to the specified directory
    pub fn save(&self) -> Result<(), LibraryError> {
        self.manifest.save_in_dir(&self.dir)?;

        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Library, LibraryError> {
        let path = path.as_ref();
        let manifest = LibManifest::load_dir(path)?;
        Ok(Library {
            dir: path.to_path_buf(),
            manifest,
        })
    }

    /// Iterates through each directory in the specified path and searches for the library manifest in it
    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Vec<Library>, LibraryError> {
        let mut entries = Vec::new();

        let read = std::fs::read_dir(dir.as_ref()).map_err(LibraryError::Io)?;

        for r in read {
            let Ok(entry) = r else {
                continue;
            };

            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            match LibManifest::load_dir(&path) {
                Ok(manifest) => {
                    entries.push(Library::new(path, manifest));
                }
                Err(LibraryError::ManifestNotFound) => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(entries)
    }
}

impl Default for Library {
    fn default() -> Self {
        Library {
            dir: "./storage".into(),
            manifest: LibManifest::builder().build(0),
        }
    }
}

/// Error interacting with manifest
#[derive(Debug)]
pub enum LibraryError {
    /// Manifest deserialization error
    DeserializationFailed,

    /// Manifest serialization error
    SerializationFailed,

    /// The manifest file at the specified path was not found
    ManifestNotFound,

    /// Invalid/Missing library configuration
    InvalidLibraryConfig,

    Io(std::io::Error),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for LibraryError {}
