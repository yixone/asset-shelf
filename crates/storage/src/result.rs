pub(crate) type Result<T> = std::result::Result<T, StorageError>;

pub enum StorageError {
    InvalidPath,
    AlreadyExists,
    InvalidSeekRange,

    Io(std::io::Error),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}
