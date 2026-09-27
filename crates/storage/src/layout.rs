use crate::path;

/// Generates storage paths for files
pub struct StorageLayout;

impl StorageLayout {
    // TODO!
}

#[allow(dead_code, reason = "Until implementing the StorageLayout")]
/// Applies path sharding, transforming: `abcdef` into `ab/cd/abcdef`
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
