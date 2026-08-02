use std::panic::Location;

type BoxDynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type StaticLocation = &'static Location<'static>;

#[derive(Debug)]
pub struct Error {
    pub(crate) kind: ErrorKind,
    pub(crate) location: StaticLocation,
}

impl Error {
    /// Returns a reference to the kind of this [`Error`]
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Creates a new [`Error`]
    #[track_caller]
    pub fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            location: Location::caller(),
        }
    }

    /// Creates a new internal [`Error`] with the specified source
    #[track_caller]
    pub fn internal<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::new(ErrorKind::Internal {
            source: Box::new(err),
        })
    }

    /// Checks whether the current [`Error`] is internal
    pub fn is_internal(&self) -> bool {
        matches!(self.kind, ErrorKind::Internal { .. })
    }
}

#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "error", content = "kind")
)]
pub enum ErrorKind {
    /// File type not supported
    UnsupportedFileType,
    /// The received file's size exceeds the maximum limit
    FileTooLarge { received: usize, max_size: usize },
    /// The file exists in the database but is missing from the storage
    FileDetached,

    /// Payload string is too long
    StringTooLong { max_size: usize },
    /// The received payload cannot be read correctly
    MalformedPayload,

    /// Pagination values ​​are too large
    PaginationTooLarge,

    /// The entity is marked as `deleted` and is `read-only` until restored
    EntityDeleted,
    /// Entity not found
    NotFound,
    /// Entity already exists
    AlreadyExists,

    /// Internal application error
    Internal {
        #[cfg_attr(feature = "serde", serde(skip))]
        source: BoxDynError,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} occurred in [{}]", self.kind, self.location)
    }
}

pub trait ResultExt<T> {
    #[track_caller]
    fn to_app_err(self) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[track_caller]
    fn to_app_err(self) -> Result<T, Error> {
        self.map_err(Error::internal)
    }
}
