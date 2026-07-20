pub type Result<T> = std::result::Result<T, StorageBackendError>;

#[derive(Debug)]
pub enum StorageBackendError {
    Io(std::io::Error),
}

impl From<std::io::Error> for StorageBackendError {
    fn from(err: std::io::Error) -> Self {
        StorageBackendError::Io(err)
    }
}

impl std::fmt::Display for StorageBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for StorageBackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageBackendError::Io(e) => Some(e),
        }
    }
}
