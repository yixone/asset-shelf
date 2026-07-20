use storage_backend::core::result::StorageBackendError;

#[derive(Debug)]
pub enum StorageError {
    BackendError(StorageBackendError),

    FileTooLarge { received: usize, excepted: usize },
    AlreadyExists,
    UnsuppotedMimetype,
}

impl<E> From<E> for StorageError
where
    StorageBackendError: From<E>,
{
    fn from(err: E) -> Self {
        StorageError::BackendError(err.into())
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::BackendError(err) => Some(err),
            _ => None,
        }
    }
}
