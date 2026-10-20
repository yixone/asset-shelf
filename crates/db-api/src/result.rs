pub(crate) type Result<T> = std::result::Result<T, DatabaseError>;

pub enum DatabaseError {
    Entity(models::EntityError),
    PaginationOverflow,
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<models::EntityError> for DatabaseError {
    fn from(err: models::EntityError) -> Self {
        match err {
            models::EntityError::Other(e) => Self::Other(e),
            _ => Self::Entity(err),
        }
    }
}

pub trait ToDatabaseError<T> {
    fn to_db_error(self) -> Result<T>;
}

impl<T, E> ToDatabaseError<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[track_caller]
    fn to_db_error(self) -> Result<T> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(DatabaseError::Other(Box::new(e))),
        }
    }
}
