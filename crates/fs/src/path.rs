use std::path::{Path, PathBuf};

use crate::result::StorageError;

const SEP_CHAR: char = '/';

/// Relative path to the file from the storage root
#[derive(Debug, Clone)]
pub struct StoragePath {
    buf: String,
}

impl StoragePath {
    /// Creates a new [`StoragePath`]
    pub fn new(st: impl Into<String>) -> Self {
        let st = st.into();
        Self { buf: st }
    }

    /// Returns [`StoragePath`] as a string slice
    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Returns [`StoragePath`] as a [`Path`]
    pub fn as_path(&self) -> &Path {
        Path::new(&self.buf)
    }

    /// Converts this [`StoragePath`] into a [PathBuf], consuming ownership
    pub fn into_path(self) -> PathBuf {
        PathBuf::from(self.buf)
    }

    /// Returns the length of this [`StoragePath`]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if the [`StoragePath`] is empty. Otherwise, returns `false`
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Extends `self` with `path`
    pub fn join(mut self, path: impl Into<String>) -> Self {
        self.buf.push(SEP_CHAR);
        self.buf.push_str(path.into().as_str());

        self
    }
}

impl AsRef<str> for StoragePath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for StoragePath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl From<StoragePath> for PathBuf {
    fn from(value: StoragePath) -> Self {
        value.into_path()
    }
}

impl std::fmt::Display for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.buf)
    }
}

/// Applies path sharding, transforming: `abcdef` into `ab/cd/abcdef`
pub fn shard(path: impl AsRef<str>, steps: usize) -> Result<String, StorageError> {
    let path = path.as_ref();

    let mut res = String::with_capacity(path.len() + steps * 3);

    // Checks that the path contains only ASCII alphanumeric characters
    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ['_', '-'].contains(&c))
    {
        return Err(StorageError::InvalidPath);
    }

    for i in 0..steps {
        let idx = 2 * i;
        if idx + 2 > path.len() {
            break;
        }
        res.push_str(&path[idx..idx + 2]);
        res.push(SEP_CHAR);
    }

    res.push_str(path);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::path::{StoragePath, shard};

    #[test]
    fn create_path() {
        let path = StoragePath::new("test")
            .join("foo")
            .join(shard("abcdef", 2).unwrap());

        assert_eq!(path.as_str(), "test/foo/ab/cd/abcdef");
    }

    #[test]
    fn into_path() {
        let path = StoragePath::new("foo").join("bar");

        assert_eq!(path.into_path(), PathBuf::from("foo/bar"));
    }

    #[test]
    fn validate_shard() {
        assert_eq!(shard("abcdef", 2).unwrap(), "ab/cd/abcdef");
        assert_eq!(shard("abc", 2).unwrap(), "ab/abc");
        assert!(shard("abc/def", 2).is_err());
        assert!(shard("../abc", 2).is_err());
        assert!(shard("💥", 2).is_err());
    }
}
