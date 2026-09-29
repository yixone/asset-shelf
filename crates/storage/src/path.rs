use crate::result::StorageError;

pub(crate) const SEP_CHAR: char = '/';

/// A path identifying the file in the storage
///
/// `StoragePath` is independent of the underlying storage backend and does not
/// represent a file system path. It identifies a stored file within
/// the application's storage namespace
#[derive(Debug, Clone, PartialEq)]
pub struct StoragePath(pub(crate) String);

impl StoragePath {
    /// Creates a new [`StoragePath`]
    pub fn new(s: impl Into<String>) -> Self {
        StoragePath(s.into())
    }

    /// Returns the storage key as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the parent path for the current path
    pub fn parent(&self) -> Option<Self> {
        self.0
            .rsplit_once(SEP_CHAR)
            .map(|(p, _)| StoragePath(p.to_owned()))
    }

    /// Returns the name of the file pointed to by the path
    pub fn file_name(&self) -> Option<&str> {
        self.0.rsplit(SEP_CHAR).next().filter(|s| !s.is_empty())
    }

    /// Sets the filename (if possible)
    pub fn set_file_name(&mut self, name: impl AsRef<str>) {
        if let Some(p) = self.parent() {
            *self = p.push(name)
        }
    }

    /// Appends a segment to the storage path
    pub fn push(&self, seg: impl AsRef<str>) -> Self {
        let mut path = self.0.clone();
        let seg = seg.as_ref();

        if !seg.starts_with(SEP_CHAR) {
            path.push(SEP_CHAR);
        }

        path.push_str(seg);

        StoragePath(path)
    }
}

impl std::fmt::Display for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for StoragePath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<StoragePath> for StoragePath {
    fn as_ref(&self) -> &StoragePath {
        self
    }
}

/// Validates a storage path for use with the filesystem
///
/// A valid path may contains only ASCII alphanumeric characters
/// and the following separators: `/`, `~` and `-`
///
/// Returns [`StorageError::InvalidPath`] if the path contains
/// unsupported characters
pub(crate) fn validate_path(path: impl AsRef<StoragePath>) -> Result<(), StorageError> {
    let path = path.as_ref().as_str();

    if path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains("//")
        || !path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '~' | '_' | '/'))
    {
        return Err(StorageError::InvalidPath);
    }

    Ok(())
}
