/// File storage error
#[derive(Debug)]
pub enum StorageError {
    /// An invalid path violating constraints was received
    InvalidPath,

    /// The file storage is mounted in read-only mode
    ReadOnly,

    /// [`std::io`] error
    Io(std::io::Error),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}
