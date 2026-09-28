pub(crate) type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug)]
pub enum StorageError {
    InvalidPath,
    AlreadyExists,
    InvalidSeekRange,
    NotFound,

    Io(std::io::Error),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => StorageError::NotFound,
            std::io::ErrorKind::AlreadyExists => StorageError::AlreadyExists,
            _ => StorageError::Io(err),
        }
    }
}
