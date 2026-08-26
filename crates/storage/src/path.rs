use std::path::PathBuf;

/// The path of the object in the storage
///
/// The [`StoragePath`] is the path to the file relative to the storage section
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoragePath(pub(crate) String);

impl StoragePath {
    /// Creates a new [`StoragePath`]
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Creates a new [`StoragePath`] with the container format:
    /// `co/nt/container/`
    pub fn new_container(container: impl Into<String>) -> Self {
        let container_dir = shard_path(&container.into(), 2);
        Self(container_dir)
    }

    /// Appends a segment to [`StoragePath`]
    pub fn join(&self, segment: impl std::fmt::Display) -> Self {
        StoragePath(format!("{}/{}", self.0, segment))
    }

    /// Returns [`StoragePath`] as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns [`StoragePath`] as [`PathBuf`]
    pub fn to_path(&self) -> PathBuf {
        PathBuf::from(&self.0)
    }
}

impl AsRef<str> for StoragePath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for StoragePath {
    type Err = InvalidStoragePathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StoragePath::new(s.to_string()))
    }
}

/// Invalid file storage path error
#[derive(Debug)]
pub struct InvalidStoragePathError;

impl std::fmt::Display for InvalidStoragePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for InvalidStoragePathError {}

/// Applies path sharding, transforming:
/// `qwerty` into `qw/er/qwerty`
fn shard_path(key: &str, levels: usize) -> String {
    let mut res = String::new();

    for i in 0..levels {
        let idx = 2 * i;
        if idx + 2 > key.len() {
            break;
        }
        res.push_str(&key[idx..idx + 2]);
        res.push('/');
    }

    res.push_str(key);
    res
}

#[cfg(test)]
mod tests {
    use flake_id::{FlakeId, str::FlakeIdStr};

    use super::*;

    #[test]
    fn build_path() {
        let path = StoragePath::new("foo").join("bar").join(5);
        assert_eq!(path.as_str(), "foo/bar/5");
    }

    #[test]
    fn build_container_path() {
        let path = StoragePath::new_container("qwerty").join(0);
        assert_eq!(path.as_str(), "qw/er/qwerty/0");
    }

    #[test]
    fn build_with_flake() {
        let flake = FlakeId(123456);
        let flake_str = FlakeIdStr("abcdef".to_string());

        let path = StoragePath::new(flake_str.to_string()).join(flake);
        assert_eq!(path.as_str(), "abcdef/123456")
    }
}
