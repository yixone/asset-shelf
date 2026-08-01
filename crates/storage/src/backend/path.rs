use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

/// The absolute path to the file in the storage
/// Abstracts the path implementation for S3 and FS
#[derive(Debug)]
pub struct StoragePath {
    /// The namespace in which the file is located
    pub(crate) namespace: String,
    /// Path to the file in the namespace
    pub(crate) key: String,
}

impl StoragePath {
    /// Creates a new [`StoragePath`]
    pub fn new<P>(namespace: P, path: P) -> Self
    where
        P: Into<String>,
    {
        Self {
            namespace: namespace.into(),
            key: path.into(),
        }
    }

    pub fn to_path(&self) -> PathBuf {
        Path::new(&self.namespace).join(&self.key)
    }
}

impl Display for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.namespace, self.key)
    }
}

impl FromStr for StoragePath {
    type Err = InvalidStoragePathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((namespace, path)) = s.split_once('/') else {
            return Err(InvalidStoragePathError);
        };
        Ok(StoragePath::new(namespace, path))
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
