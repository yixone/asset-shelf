use std::{fmt::Display, path::PathBuf, str::FromStr};

/// The absolute path to the file in the storage
/// Abstracts the path implementation for S3 and FS
#[derive(Debug, Clone)]
pub struct StoragePath {
    /// Path to the file in the namespace
    pub(crate) key: String,
}

impl StoragePath {
    /// Creates a new [`StoragePath`]
    pub fn new<P>(path: P) -> Self
    where
        P: Into<String>,
    {
        Self { key: path.into() }
    }

    pub fn join(&self, segment: &str) -> Self {
        StoragePath {
            key: format!("{}/{}", self.key, segment),
        }
    }

    pub fn to_path(&self) -> PathBuf {
        PathBuf::from(&self.key)
    }
}

impl Display for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl FromStr for StoragePath {
    type Err = InvalidStoragePathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StoragePath::new(s.to_string()))
    }
}

#[derive(Debug)]
pub struct InvalidStoragePathError;

impl std::fmt::Display for InvalidStoragePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for InvalidStoragePathError {}
