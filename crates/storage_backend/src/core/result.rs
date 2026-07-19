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
