use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct StoragePath {
    pub namespace: String,
    pub key: String,
}

impl StoragePath {
    pub fn as_path(&self) -> PathBuf {
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
        let Some((namespace, key)) = s.split_once('/') else {
            return Err(InvalidStoragePathError);
        };
        Ok(StoragePath {
            namespace: namespace.to_string(),
            key: key.to_string(),
        })
    }
}

impl From<StoragePath> for PathBuf {
    fn from(val: StoragePath) -> Self {
        Path::new(&val.namespace).join(&val.key)
    }
}

impl From<&StoragePath> for PathBuf {
    fn from(val: &StoragePath) -> Self {
        Path::new(&val.namespace).join(&val.key)
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
