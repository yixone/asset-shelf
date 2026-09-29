use storage_types::StoragePath;

use crate::result::{Result, StorageError};

/// Creates a temporary path based on the specified storage path
///
/// The original file name is prefixed with `~` and suffixed with random
/// identifier to produce an unique temporary file name
///
/// Returns [`StorageError::InvalidPath`] if the path does not contain a file name
pub fn make_temp_path(mut p: StoragePath) -> Result<StoragePath> {
    let temp_name = p
        .file_name()
        .map(|n| format!("~{}-{}", n, base62::random_str(6)))
        .ok_or(StorageError::InvalidPath)?;

    p.set_file_name(temp_name);
    Ok(p)
}

/// Validates a storage path for use with the filesystem
///
/// A valid path may contains only ASCII alphanumeric characters
/// and the following separators: `/`, `~` and `-`
///
/// Returns [`StorageError::InvalidPath`] if the path contains
/// unsupported characters
pub(crate) fn validate_path(path: impl AsRef<StoragePath>) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_temp_path() {
        let path = StoragePath::new("test").push("file");
        let temp = make_temp_path(path).unwrap();

        assert!(temp.file_name().unwrap().starts_with("~file-"));
    }
}
