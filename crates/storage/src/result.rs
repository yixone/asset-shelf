use storage_backend::core::result::StorageBackendError;

#[derive(Debug)]
pub enum StorageError {
    BackendErr(StorageBackendError),
    FileTooLarge { received: usize, excepted: usize },
    AlreadyExists,
    UnsuppotedMimetype,
}

impl<E> From<E> for StorageError
where
    StorageBackendError: From<E>,
{
    fn from(err: E) -> Self {
        StorageError::BackendErr(err.into())
    }
}
