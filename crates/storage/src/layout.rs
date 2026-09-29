use schema::{id::AssetId, types::MediaVariant};

use crate::path::{self, StoragePath};

/// Generates storage paths for AssetShelf files
pub struct StorageLayout;

impl StorageLayout {
    /// Returns the storage path for a media variant
    ///
    /// Media files are grouped under `media` and sharded by
    /// the first two characters of the asset ID
    pub fn media(id: impl AsRef<AssetId>, variant: MediaVariant) -> StoragePath {
        StoragePath::new("media")
            .push(shard(id.as_ref().to_string(), 2))
            .push(variant.as_str())
    }
}

/// Applies path sharding, transforming: `abcdef` into `ab/cd/abcdef`
///
/// The input value must contain only ASCII characters
fn shard(value: impl AsRef<str>, steps: usize) -> String {
    let path = value.as_ref();

    let mut res = String::with_capacity(path.len() + steps * 3);

    for i in 0..steps {
        let idx = 2 * i;
        if idx + 2 > path.len() {
            break;
        }
        res.push_str(&path[idx..idx + 2]);
        res.push(path::SEP_CHAR);
    }

    res.push_str(path);
    res
}
