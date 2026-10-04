pub(crate) type Result<T> = std::result::Result<T, EntityError>;

pub enum EntityError {
    NotFound,
    BadInput,
    AlreadyExists,
    Deleted,
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub trait ToEntityError<T> {
    fn to_entity_error(self) -> Result<T>;
}

impl<T, E> ToEntityError<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[track_caller]
    fn to_entity_error(self) -> Result<T> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(EntityError::Other(Box::new(e))),
        }
    }
}
