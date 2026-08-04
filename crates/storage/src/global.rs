use crate::{StoragePath, backend::StorageBackend};

/// The part of the storage system responsible for storing persistent files
pub(crate) struct GlobalSection {
    /// File storage backend
    pub backend: Box<dyn StorageBackend>,
}

/// Data for global path generation
pub struct GlobalPathData<'a> {
    /// Container name
    container: &'a str,
    /// File path within the container
    file: &'a str,
}

impl<'a> GlobalPathData<'a> {
    /// Creates a new [`GlobalPathData`]
    pub fn new(container: &'a str, file: &'a str) -> Self {
        Self { container, file }
    }
}

/// Generates a global path for a file in persistent storage section
pub(crate) fn generate_global_path(data: GlobalPathData<'_>) -> StoragePath {
    let container = shard_key(data.container, 2);
    StoragePath::new(container).join(data.file)
}

/// Applies path sharding, transforming:
/// `qwerty` into `qw/er/qwerty`
fn shard_key(key: &str, levels: usize) -> String {
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
